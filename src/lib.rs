use thiserror::Error;

mod buffer;
mod listener;

const MAX_MSG_LEN: usize = 0xFFFF;

#[derive(Error, Debug)]
pub enum Error {
    #[error("exceeded write ptr beyond read")]
    BufferWriteOverflow,
}

/// The protocol specifies that each message contains
/// a static magic byte, the data length, and the data
/// itself.
///
/// A message is deemed invalid if the data inside of the message
/// does not match header length specified
#[derive(Clone, PartialEq)]
pub struct Ctmp {
    length: u16,
    pub data: [u8; MAX_MSG_LEN],
}

impl Ctmp {
    #[inline(always)]
    pub fn new(valid_len: u16, data: [u8; MAX_MSG_LEN]) -> Self {
        Ctmp {
            length: valid_len,
            data,
        }
    }
}

impl Default for Ctmp {
    fn default() -> Self {
        Ctmp {
            length: 0,
            data: [0; MAX_MSG_LEN],
        }
    }
}
