use thiserror::Error;

mod buffer;
mod listener;

#[derive(Error, Debug)]
pub enum Error {
    #[error("exceeded write ptr beyond read")]
    BufferWriteOverflow,
}
