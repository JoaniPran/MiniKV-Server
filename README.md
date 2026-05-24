# MiniKV

MiniKV es un pequeño proyecto en Rust que simula un almacén clave-valor (similar a un HashMap) en memoria RAM y permite operaciones en tiempo real a través de un cliente y un servidor.

## Binarios

El repositorio contiene dos binarios ejecutables:

- `minikv-server`: servidor que mantiene el estado en memoria y responde a comandos del cliente.
- `minikv-client`: cliente que envía comandos al servidor.

## Requisitos

- Rust y Cargo instalados. (https://www.rust-lang.org)

## Ejecutar

Arrancar el servidor (ejemplo en localhost puerto 3000):

```
cargo run --bin minikv-server -- 127.0.0.1:3000
```

Iniciar el cliente (conecta al servidor en 127.0.0.1:3000):

```
cargo run --bin minikv-client -- 127.0.0.1:3000
```

## Comandos disponibles

Desde el cliente se pueden utilizar los siguientes comandos:

- `set <clave> <valor>`: Inserta o actualiza la clave con el valor indicado.
- `set <clave>`: Si se invoca sin valor, elimina la clave.
- `get <clave>`: Devuelve el valor asociado a la clave.
- `length`: Devuelve el número de claves almacenadas en el servidor.
- `snapshot`: Fuerza al servidor a escribir el estado/persistencia en disco.

Ejemplos:

```
set foo bar
get foo
length
set foo         # elimina la clave 'foo'
snapshot
```

## Archivos importantes

- `Cargo.toml` — definición del paquete y dependencias.
- `src/` — código fuente del servidor y cliente.
- `.minikv.data` — archivo de persistencia (snapshot) generado por el servidor.
- `.minikv.log` — archivo de log de operaciones.

----


# MiniKV-Server
