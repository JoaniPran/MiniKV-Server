use crate::error::ErrType;
use crate::minikv::Minikv;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub fn check_args(tamanio: usize, esperado: usize) -> Result<(), ErrType> {
    if tamanio != esperado {
        report_error(&ErrType::InvalidArgs);
        return Err(ErrType::InvalidArgs);
    }
    Ok(())
}

pub fn get_direccion(arg: Option<&String>) -> Result<&String, ErrType> {
    match arg {
        Some(direccion) => Ok(direccion),
        None => {
            report_error(&ErrType::InvalidArgs);
            Err(ErrType::InvalidArgs)
        }
    }
}

pub fn check_socket(resultado_bind: std::io::Result<TcpListener>) -> Result<TcpListener, ErrType> {
    match resultado_bind {
        Ok(l) => Ok(l),
        Err(_) => {
            report_error(&ErrType::ServerSocketBinding);
            Err(ErrType::ServerSocketBinding)
        }
    }
}

pub fn check_db(resultado: Result<Minikv, ErrType>) -> Result<Arc<Mutex<Minikv>>, ErrType> {
    match resultado {
        Ok(instancia) => Ok(Arc::new(Mutex::new(instancia))),
        Err(e) => {
            report_error(&e); // Ya reportamos el error específico (Data o Log file)
            Err(e)
        }
    }
}

pub fn check_client_connect(
    res: std::io::Result<std::net::TcpStream>,
) -> Result<std::net::TcpStream, ErrType> {
    match res {
        Ok(s) => Ok(s),
        Err(_) => {
            report_error(&ErrType::ClientSocketBinding);
            Err(ErrType::ClientSocketBinding)
        }
    }
}

pub fn configure_timeouts(
    socket: &std::net::TcpStream,
    read_ms: u64,
    write_ms: u64,
) -> Result<(), ErrType> {
    let read_res = socket.set_read_timeout(Some(std::time::Duration::from_millis(read_ms)));
    let write_res = socket.set_write_timeout(Some(std::time::Duration::from_millis(write_ms)));

    if read_res.is_err() || write_res.is_err() {
        // Si falla configurar el socket del SO, es un error de comunicación/binding
        report_error(&ErrType::ClientSocketBinding);
        return Err(ErrType::ClientSocketBinding);
    }
    Ok(())
}

pub fn report_error(e: &ErrType) {
    println!("{}", mapear_error(e));
}

pub fn mapear_error(e: &ErrType) -> String {
    let motivo = match e {
        ErrType::NotFound => "NOT FOUND",
        ErrType::ExtraArgument => "EXTRA ARGUMENT",
        ErrType::MissingArgument => "MISSING ARGUMENT",
        ErrType::UnknownCommand => "UNKNOWN COMMAND",
        ErrType::ConnectionClosed => "CONNECTION CLOSED",
        ErrType::InternalServerError => "INTERNAL SERVER ERROR",
        ErrType::Timeout => "TIMEOUT",
        ErrType::ClientSocketBinding => "CLIENT SOCKET BINDING",
        _ => "INTERNAL ERROR",
    };
    format!("ERROR \"{}\"", motivo)
}
