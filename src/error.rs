#[derive(Debug)]
pub enum ErrType {
    NotFound,
    ExtraArgument,
    InvalidDataFile,
    InvalidLogFile,
    MissingArgument,
    UnknownCommand,
    ConnectionClosed,
    InternalServerError,
    InvalidArgs,
    ServerSocketBinding,
    Timeout,
    ClientSocketBinding,
}
