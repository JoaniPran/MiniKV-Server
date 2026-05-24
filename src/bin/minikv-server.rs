use minikv_server_01::error::ErrType;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use minikv_server_01::{
    comands::ejecutar_comando, errors_connection::check_args, errors_connection::check_db,
    errors_connection::check_socket, errors_connection::get_direccion,
    errors_connection::mapear_error, errors_connection::report_error, minikv::Minikv,
};

fn main() {
    // 1. Validar argumentos del servidor
    let args: Vec<String> = std::env::args().collect();

    if check_args(args.len(), 2).is_err() {
        return;
    }

    let Ok(direccion) = get_direccion(args.get(1)) else {
        return;
    };

    let Ok(db) = check_db(Minikv::new()) else {
        return;
    };

    // 3. Bindeo del Socket de Escucha El S.O. crea un Socket de escucha. con esa direccion y port
    let Ok(listener) = check_socket(TcpListener::bind(direccion)) else {
        return;
    };
    println!("Servidor MiniKV escuchando en {}", direccion);

    attend_clients(listener, db);
}

pub fn attend_clients(listener: TcpListener, db: Arc<Mutex<Minikv>>) {
    // el mutex me permite utilizar el lock y bloquear la bdd para que cuando un hilo escriba se bloquee
    // 4. Loop de Aceptación: Por cada cliente, un hilo nuevo, en stream tenemos una conxion seguro entre el cliente y el server
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let db_clone = Arc::clone(&db);
                thread::spawn(move || {
                    if manejar_cliente(&mut stream, db_clone).is_err() {
                        // Si hay error de comunicación, imprimimos y cerramos el hilo
                        report_error(&ErrType::ConnectionClosed);
                    }
                });
            }
            Err(_) => report_error(&ErrType::ConnectionClosed),
        }
    }
}

/// Función que gestiona la sesión de un cliente específico
fn manejar_cliente(
    stream: &mut std::net::TcpStream,
    db: Arc<Mutex<Minikv>>,
) -> std::io::Result<()> {
    // Clonamos el stream para poder leer y escribir de forma independiente si fuera necesario
    let reader: BufReader<std::net::TcpStream> = BufReader::new(stream.try_clone()?);

    procesar_comandos(reader, stream, db)?;

    Ok(())
}

fn procesar_comandos(
    reader: BufReader<std::net::TcpStream>,
    stream: &mut std::net::TcpStream,
    db: Arc<Mutex<Minikv>>,
) -> std::io::Result<()> {
    for linea in reader.lines() {
        let linea_recibida = linea?;

        // 2. Parseo de la entrada
        let Ok(partes) = minikv_server_01::minikv::parse_quoted(
            &linea_recibida,
            minikv_server_01::error::ErrType::UnknownCommand,
        ) else {
            enviar_respuesta(
                stream,
                &mapear_error(&minikv_server_01::error::ErrType::UnknownCommand),
            )?;
            continue;
        };

        if partes.is_empty() {
            continue;
        }

        let respuesta = atender_operacion(&db, &partes);

        enviar_respuesta(stream, &respuesta)?;
    }

    Ok(())
}

fn atender_operacion(db: &Arc<Mutex<Minikv>>, partes: &[String]) -> String {
    let Ok(mut kv) = db.lock() else { // el lock solo bloquea el hilo hasta que este termine, El lock bloquea el acceso a todo el recurso (la base de datos completa)
        return mapear_error(&ErrType::InternalServerError);
    };

    match ejecutar_comando(&mut kv, partes) {
        Ok(resultado) => resultado,
        Err(e) => mapear_error(&e),
    }
}

fn enviar_respuesta(stream: &mut std::net::TcpStream, mensaje: &str) -> std::io::Result<()> {
    stream.write_all(format!("{}\n", mensaje).as_bytes())
}
