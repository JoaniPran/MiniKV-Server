use std::fs::OpenOptions;
use std::io::Write;

use crate::error::ErrType;

/// Persiste una clave y su valor en el almacenamiento físico (disco).
///
/// Esta función maneja la escritura tanto para archivos de log como para archivos de datos,
/// aplicando un formato específico según la extensión del archivo y escapando caracteres especiales.
///
/// # Formato de salida
/// - Si el archivo termina en `.log`:
/// - Si el archivo no termina en `.log` (asumido como `.data`):
///
///
/// # Escapado de caracteres
/// La función reemplaza las comillas dobles (`"`) por comillas escapadas (`\"`) para asegurar
/// que el parseo posterior sea consistente y no se rompa con valores que contengan comillas.
///
/// # Argumentos
/// * `clave` - La clave a persistir.
/// * `valor` - El valor asociado a la clave (puede estar vacío para representar un borrado).
/// * `file` - La ruta del archivo donde se realizará la escritura.
///
pub fn save_in_the_disk(clave: &str, valor: &str, file: &str) -> Result<(), ErrType> {
    let mut archivo = OpenOptions::new()
        .create(true) //lo creo si no existe el .log
        .append(true) // si existe lo abro
        .open(file) // ejecuto abrir o crear dependiendo si existe o no
        .map_err(|_| {
            if file.ends_with(".log") {
                ErrType::InvalidLogFile
            } else {
                ErrType::InvalidDataFile
            }
        })?;

    let c_safe = clave.replace("\"", "\\\""); // le dice busca la comilla suelta (") con (\") por que rust lee asi las comillas y ponele una (\\) -> asi se escriben las barras en rust  + (\") y asi las comillas
    let v_safe = valor.replace("\"", "\\\"");

    if file.ends_with(".log") {
        if valor.is_empty() {
            writeln!(archivo, "set \"{}\"", c_safe).map_err(|_| ErrType::InvalidLogFile)?;
        } else {
            writeln!(archivo, "set \"{}\" \"{}\"", c_safe, v_safe)
                .map_err(|_| ErrType::InvalidLogFile)?;
        }
    } else {
        writeln!(archivo, "\"{}\" \"{}\"", c_safe, v_safe).map_err(|_| ErrType::InvalidDataFile)?;
    }
    Ok(())
}

mod tests {
    #[cfg(test)]
    use std::fs::File;
    #[cfg(test)]
    use std::io::{BufRead, BufReader};

    #[test]
    fn test_save_data() -> Result<(), super::ErrType> {
        let path = "test_minikv.data";

        super::save_in_the_disk("hola \"mundo\"", "chau \"mundo\"", path)?;

        // Abrimos el archivo y buscamos la línea sin cargar todo en memoria
        let file = File::open(path).map_err(|_| super::ErrType::InvalidDataFile)?;
        let reader = BufReader::new(file);

        let mut found = false;
        for linea in reader.lines() {
            let l = linea.map_err(|_| super::ErrType::InvalidDataFile)?;
            if l.contains("\"hola \\\"mundo\\\"\" \"chau \\\"mundo\\\"\"") {
                found = true;
                break;
            }
        }
        assert!(
            found,
            "El dato guardado no coincide con el formato esperado"
        );

        std::fs::remove_file(path).map_err(|_| super::ErrType::InvalidDataFile)?;
        Ok(())
    }

    #[test]
    fn test_save_in_the_disk_log() -> Result<(), super::ErrType> {
        let file_path = "test_unitario.log";
        let _ = std::fs::remove_file(file_path);

        super::save_in_the_disk("k", "v", file_path).map_err(|_| super::ErrType::InvalidLogFile)?;

        // Leemos la primera línea para comparar el contenido del log
        let file = File::open(file_path).map_err(|_| super::ErrType::InvalidLogFile)?;
        let mut lines = BufReader::new(file).lines();

        let primera_linea = lines
            .next()
            .ok_or(super::ErrType::InvalidLogFile)?
            .map_err(|_| super::ErrType::InvalidLogFile)?;

        assert_eq!(primera_linea.trim(), "set \"k\" \"v\"");

        std::fs::remove_file(file_path).map_err(|_| super::ErrType::InvalidLogFile)?;
        Ok(())
    }
}
