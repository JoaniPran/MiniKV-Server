use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::write_disk::save_in_the_disk;

use crate::error::ErrType;

/// Estructura principal de la base de datos clave-valor.
///
/// Mantiene un `HashMap` en memoria para acceso rápido y gestiona
/// la persistencia a través de archivos de log y data.
pub struct Minikv {
    pub mi_kvs: HashMap<String, String>,
}

impl Default for Minikv {
    /// Provee una instancia por defecto.
    /// Intenta cargar archivos existentes y, si falla, devuelve una base de datos vacía.
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Minikv {
            mi_kvs: HashMap::new(),
        })
    }
}

impl Minikv {
    /// Crea una nueva instancia de `Minikv` cargando los archivos de persistencia.
    ///
    /// Primero intenta cargar el archivo `.minikv.data` (snapshot) y luego
    /// aplica las operaciones pendientes del archivo `.minikv.log`.
    ///
    /// # Errors
    /// Devuelve `ErrType` si alguno de los archivos existe pero tiene un formato inválido.
    pub fn new() -> Result<Self, ErrType> {
        let mut mapa: HashMap<String, String> = HashMap::new();

        if let Ok(archivo_data) = fs::File::open(".minikv.data") {
            load_data_file(&mut mapa, &archivo_data)?;
        }

        if let Ok(archivo_log) = fs::File::open(".minikv.log") {
            load_log_file(&mut mapa, &archivo_log)?;
        }

        Ok(Minikv { mi_kvs: mapa })
    }
}
/// Inserta una clave y un valor en el mapa en memoria.
impl Minikv {
    pub fn insert_key_and_value(&mut self, clave: &String, valor: &String) {
        self.mi_kvs.insert(clave.to_string(), valor.to_string());
    }
}
/// Elimina una clave del mapa en memoria.
impl Minikv {
    pub fn remove(&mut self, clave: &String) {
        self.mi_kvs.remove(clave);
    }
}
/// Busca un valor por clave e imprime su resultado.
///
/// # Errors
/// Devuelve `ErrType::NotFound` si la clave no existe en la base de datos.
impl Minikv {
    pub fn get(&mut self, clave: &String) -> Result<String, ErrType> {
        match self.mi_kvs.get(clave) {
            Some(valor) => Ok(valor.to_string()),
            None => Err(ErrType::NotFound),
        }
    }
}

/// Realiza un snapshot de la base de datos actual.
///
/// Escribe todo el contenido de la memoria en `data_file` y vacía el `log_file`.
///
/// # Arguments
/// * `data_file` - Ruta del archivo donde se guardará el estado actual.
/// * `log_file` - Ruta del archivo de log que será reiniciado.
///
/// # Errors
/// Devuelve `ErrType::InvalidDataFile` o `ErrType::InvalidLogFile` si falla la escritura.
impl Minikv {
    pub fn write_data(&mut self, data_file: &str, log_file: &str) -> Result<(), ErrType> {
        File::create(data_file).map_err(|_| ErrType::InvalidDataFile)?;

        for (clave, valor) in &self.mi_kvs {
            save_in_the_disk(clave, valor, data_file).map_err(|_| ErrType::InvalidDataFile)?;
        }

        File::create(log_file).map_err(|_| ErrType::InvalidLogFile)?;

        Ok(())
    }
}

/// Desescapa una cadena que contiene secuencias de escape simples (solo \\ y \").
pub fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                result.push(next);
            }
        } else {
            result.push(c);
        }
    }
    result
}
pub fn parse_quoted(linea: &str, err: ErrType) -> Result<Vec<String>, ErrType> {
    let mut partes = Vec::new();
    let mut actual = String::new();
    let (mut esc, mut dentro, mut started) = (false, false, false);
    for c in linea.chars() {
        if esc {
            actual.push(c);
            esc = false;
            started = true
        } else if c == '\\' {
            esc = true
        } else if c == '"' {
            dentro = !dentro;
            started = true
        } else if dentro {
            actual.push(c)
        } else if c.is_whitespace() {
            if !actual.is_empty() || started {
                partes.push(std::mem::take(&mut actual));
                started = false
            }
        } else {
            actual.push(c);
            started = true
        }
    }
    error(dentro, esc, err)?;
    push_string(&mut partes, actual, started);
    Ok(partes)
}

pub fn error(dentro: bool, esc: bool, err: ErrType) -> Result<(), ErrType> {
    if dentro || esc { Err(err) } else { Ok(()) }
}

pub fn push_string(partes: &mut Vec<String>, actual: String, started: bool) {
    if !actual.is_empty() || started {
        partes.push(actual);
    }
}

/// Carga el archivo de snapshot (.data) en el mapa de memoria.
fn load_data_file(mapa: &mut HashMap<String, String>, file: &File) -> Result<(), ErrType> {
    for linea in BufReader::new(file).lines() {
        let linea = linea.map_err(|_| ErrType::InvalidDataFile)?;
        if linea.is_empty() {
            continue;
        }

        let partes = parse_quoted(&linea, ErrType::InvalidDataFile)?;

        if partes.len() != 2 {
            return Err(ErrType::InvalidDataFile);
        }

        if let (Some(clave), Some(valor)) = (partes.first(), partes.get(1)) {
            mapa.insert(unescape(clave), unescape(valor));
        }
    }
    Ok(())
}

/// Carga el archivo de log (.log) aplicando las operaciones secuencialmente.
///
/// Cada línea debe comenzar con el comando `set`. Si el valor está vacío,
/// la clave se elimina del mapa.
fn load_log_file(mapa: &mut HashMap<String, String>, file: &File) -> Result<(), ErrType> {
    for linea in BufReader::new(file).lines() {
        let l = linea.map_err(|_| ErrType::InvalidLogFile)?;
        if l.trim().is_empty() {
            continue;
        }
        if !l.starts_with("set") {
            return Err(ErrType::InvalidLogFile);
        }

        let partes = parse_quoted(&l[3..], ErrType::InvalidLogFile)?;

        match &partes[..] {
            [k, v] if !v.is_empty() => {
                mapa.insert(unescape(k), unescape(v));
            }
            [k, _] | [k] => {
                mapa.remove(&unescape(k));
            }
            _ => return Err(ErrType::InvalidLogFile),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() -> Result<(), ErrType> {
        let mut kv = Minikv::new()?;
        let data_path = "test_snapshot.data";
        let log_path = "test_snapshot.log";

        kv.insert_key_and_value(&"user".to_string(), &"admin".to_string());
        kv.write_data(data_path, log_path)?;

        let file = File::open(data_path).map_err(|_| ErrType::InvalidDataFile)?;
        let reader = BufReader::new(file);

        let mut found = false;
        for linea in reader.lines() {
            let l = linea.map_err(|_| ErrType::InvalidDataFile)?;
            if l.contains("\"user\" \"admin\"") {
                found = true;
                break;
            }
        }

        assert!(
            found,
            "El archivo de snapshot no contiene los datos esperados"
        );

        let log_metadata = fs::metadata(log_path).map_err(|_| ErrType::InvalidLogFile)?;
        assert_eq!(log_metadata.len(), 0);

        fs::remove_file(data_path).map_err(|_| ErrType::InvalidDataFile)?;
        fs::remove_file(log_path).map_err(|_| ErrType::InvalidLogFile)?;
        Ok(())
    }
}
