use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

use minikv_server_01::error::ErrType;

use minikv_server_01::{
    errors_connection::check_args, errors_connection::check_client_connect,
    errors_connection::configure_timeouts, errors_connection::get_direccion,
    errors_connection::report_error,
};

// Configuración de Timeouts constante
const READ_TIMEOUT_MS: u64 = 5000; // 5 segundos
const WRITE_TIMEOUT_MS: u64 = 2000; // 2 segundos

fn main() {
    let args: Vec<String> = env::args().collect();

    if check_args(args.len(), 2).is_err() {
        return;
    }

    let Ok(address) = get_direccion(args.get(1)) else {
        return;
    };

    let Ok(socket) = check_client_connect(TcpStream::connect(address)) else {
        return;
    };

    if configure_timeouts(&socket, READ_TIMEOUT_MS, WRITE_TIMEOUT_MS).is_err() {
        return;
    }

    // 3. Orquestación
    iniciar_bucle_operaciones(socket);
}

fn iniciar_bucle_operaciones(socket: TcpStream) {
    let Ok(reader_stream) = socket.try_clone() else {
        report_error(&ErrType::ConnectionClosed);
        return;
    };

    let mut reader = BufReader::new(reader_stream);
    let mut socket_writer = socket;
    let stdin = std::io::stdin();

    for line in stdin.lock().lines() {
        let Ok(input) = line else { break };

        if input.trim().is_empty() {
            continue;
        }

        if ejecutar_intercambio(&mut socket_writer, &mut reader, input).is_err() {
            break;
        }
    }
}

fn ejecutar_intercambio(
    writer: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    mensaje: String,
) -> Result<(), ()> {
    if writer
        .write_all(format!("{}\n", mensaje).as_bytes())
        .is_err()
    {
        report_error(&ErrType::ConnectionClosed);
        return Err(());
    }

    let mut response = String::new();
    match reader.read_line(&mut response) {
        Ok(0) => {
            report_error(&ErrType::ConnectionClosed);
            Err(())
        }
        Ok(_) => {
            print!("{}", response);
            Ok(())
        }
        Err(e) => {
            handle_network_error(e);
            Err(())
        }
    }
}

fn handle_network_error(e: std::io::Error) {
    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut {
        report_error(&ErrType::Timeout);
    } else {
        report_error(&ErrType::ConnectionClosed);
    }
}
