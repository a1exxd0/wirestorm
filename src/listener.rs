use std::{
    collections::VecDeque,
    io::Read,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

use crate::{Error, buffer::MessageMgr};

const LISTENER_PORT: u16 = 33333;

/// Specifically single client implementation
pub struct CtmpListener {
    tcp: TcpListener,
    mgr: MessageMgr,
    /// terminate
    term: Arc<AtomicBool>,
    tcp_buf: VecDeque<u8>,
}

enum HeaderType {
    Partial,
    Invalid,
    Complete(u16),
}

impl CtmpListener {
    #[allow(dead_code)]
    pub fn start(
        mgr: MessageMgr,
        term: Arc<AtomicBool>,
    ) -> Result<JoinHandle<Result<(), Error>>, Error> {
        tracing::info!("building ctmp listener");
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, LISTENER_PORT))
            .map_err(Error::SocketListenerFaliure)?;

        let listener = Self {
            tcp: listener,
            mgr,
            term,
            tcp_buf: VecDeque::new(),
        };

        Ok(listener.listen())
    }

    fn listen(mut self) -> JoinHandle<Result<(), Error>> {
        thread::spawn(move || -> Result<(), Error> {
            let mut stream_addr = None;
            while !self.term.load(Ordering::Relaxed) {
                if stream_addr.is_none() {
                    tracing::info!("attempting to accept tcp connection");
                    let (s, a) = self.tcp.accept().map_err(Error::ClientAcceptError)?;

                    tracing::info!(client_addr = ?a, "established tcp connection");

                    stream_addr = Some((s, a));
                } else {
                    let mut temp_buf = [0; 0x1000];
                    let (stream, addr) = stream_addr.as_mut().unwrap();
                    match stream.read(&mut temp_buf) {
                        Ok(0) => {
                            tracing::info!(client_addr = ?addr, "connection closed");
                        }
                        Ok(n) => {
                            self.tcp_buf.extend(&temp_buf[..n]);
                        }
                        Err(e) => return Err(Error::TcpReadError(e)),
                    }
                }
            }

            tracing::info!("ctmp listener recieved termination signal");
            match stream_addr.as_ref() {
                Some(stream_addr) => {
                    tracing::info!(client_addr = ?stream_addr.1, "found live connection, shutting down");
                    stream_addr
                        .0
                        .shutdown(std::net::Shutdown::Both)
                        .map_err(Error::ClientAcceptError)?;
                }
                None => (),
            }

            tracing::info!("ctmp listener successfully terminated");
            Ok(())
        })
    }

    /// Parse tcp buffer. Rules:
    ///
    /// Assume we have a pointer to the start of our
    /// message buffer. Then:
    /// - Find the first valid header from the start
    ///   (this could skip all erroneous bytes in
    ///   buffer). If there are < 8 bytes in the buffer
    ///   then check that whatever bytes are in there
    ///   are valid
    /// - If our valid header has valid data length
    ///   add this to list of returned messages.
    /// - If it is too short (when the buffer length
    ///   is less than the data we want), wait for more
    ///   bytes.
    /// - If it is too long (the following bytes don't
    ///   form a valid header) then drop the message
    /// - Loop until we reach empty buffer or too short.
    fn parse_all(&mut self) -> Vec<bytes::Bytes> {
        todo!()
    }
}

fn parse_ctmp_header(buf: &VecDeque<u8>) -> HeaderType {
    match buf.len() {
        0 => HeaderType::Partial,
        1 if buf[0] == 0xCC => HeaderType::Partial,
        2..=7 if buf[0] == 0xCC && buf[1] == 0 && buf.iter().skip(4).all(|&b| b == 0) => {
            HeaderType::Partial
        }
        len if len >= 8
            && buf[0] == 0xCC
            && buf[1] == 0
            && buf.iter().skip(4).take(4).all(|&b| b == 0) =>
        {
            HeaderType::Complete(u16::from_be_bytes([buf[2], buf[3]]))
        }
        _ => HeaderType::Invalid,
    }
}
