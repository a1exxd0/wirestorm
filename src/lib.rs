use std::io;

use thiserror::Error;

mod buffer;
mod listener;

#[derive(Error, Debug)]
pub enum Error {
    #[error("exceeded write ptr beyond read")]
    BufferWriteOverflow,
    #[error("failed to establish socket connection: {0}")]
    SocketListenerFaliure(io::Error),
    #[error("failed to accept any incoming tcp req: {0}")]
    ClientAcceptError(io::Error),
    #[error("failed to read tcp message: {0}")]
    TcpReadError(io::Error),
}
