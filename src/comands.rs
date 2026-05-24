const FILE_NAME_LOG: &str = ".minikv.log";
const FILE_NAME_DATA: &str = ".minikv.data";

use crate::error::ErrType;
use crate::minikv::Minikv as other_minikv;
use crate::write_disk::save_in_the_disk;

pub fn ejecutar_comando(db: &mut other_minikv, args: &[String]) -> Result<String, ErrType> {
    // El primer argumento ahora suele ser el comando directamente desde el socket
    let cmd = args.first().ok_or(ErrType::UnknownCommand)?;
    let len = args.len();

    match cmd.as_str() {
        "set" => handle_set(db, args),
        "get" => {
            check_args(len, 2)?; // get <clave> -> 2 argumentos
            let clave = args.get(1).ok_or(ErrType::MissingArgument)?;
            db.get(clave) // Este get ahora debe retornar Result<String, ErrType>
        }
        "length" => {
            check_args(len, 1)?;
            Ok(db.mi_kvs.len().to_string())
        }
        "snapshot" => {
            check_args(len, 1)?;
            db.write_data(FILE_NAME_DATA, FILE_NAME_LOG)?;
            Ok("OK".to_string())
        }
        _ => Err(ErrType::UnknownCommand),
    }
}

pub fn check_args(len: usize, esperado: usize) -> Result<(), ErrType> {
    if len < esperado {
        return Err(ErrType::MissingArgument);
    }
    if len > esperado {
        return Err(ErrType::ExtraArgument);
    }
    Ok(())
}

fn handle_set(db: &mut other_minikv, args: &[String]) -> Result<String, ErrType> {
    if args.len() > 3 {
        return Err(ErrType::ExtraArgument);
    }

    let clave = args.get(1).ok_or(ErrType::MissingArgument)?;

    match args.get(2) {
        Some(valor) => {
            save_in_the_disk(clave, valor, FILE_NAME_LOG).map_err(|_| ErrType::InvalidLogFile)?;
            db.insert_key_and_value(clave, valor);
        }
        None => {
            save_in_the_disk(clave, "", FILE_NAME_LOG).map_err(|_| ErrType::InvalidLogFile)?;
            db.remove(clave);
        }
    }

    Ok("OK".to_string())
}
