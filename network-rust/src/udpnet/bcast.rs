use cbc::RecvError;
use cbc::SendError;
use crossbeam_channel as cbc;
use log::warn;
use serde::Deserialize;
use socket2::Socket;

use std::error;
use std::str;
use std::io;

#[path = "./sock.rs"]
mod sock;

#[derive(Debug)]
pub enum BcError<T> {
    IOError(io::Error),
    CBCSendError(SendError<T>),
    CBCRecvError(RecvError)
}


impl<T> From<io::Error> for BcError<T> {
    fn from(e: io::Error) -> Self {
        BcError::IOError(e)
    }
}

impl<T> From<SendError<T>> for BcError<T> {
    fn from(e: SendError<T>) -> Self {
        BcError::CBCSendError(e)
    }
}

impl<T> From<RecvError> for BcError<T> {
    fn from(e: RecvError) -> Self {
        BcError::CBCRecvError(e)
    }
}

pub fn tx<T: serde::Serialize>(port: u16, ch: cbc::Receiver<T>) -> Result<(), BcError<T>> {
    let (s, addr) = sock::new_tx(port)?;
    loop {
        let data = ch.recv()?;
        let serialized = serde_json::to_string(&data).unwrap();
        if let Err(e) = s.send_to(serialized.as_bytes(), &addr) {
            warn!("Unable to send packet, {}", e);
        }
    }
}

pub fn rx<T: serde::de::DeserializeOwned>(port: u16, ch: cbc::Sender<T>) -> Result<(), BcError<T>> {
    let s = sock::new_rx(port)?;

    let mut buf = [0; 1024];

    loop {
        match parse_packet(&s, &mut buf) {
            Ok(d) => ch.send(d)?,
            Err(e) => warn!("Received bad package got error: {}", e),
        }
    }
}

fn parse_packet<'a, T: Deserialize<'a>>(
    s: &'_ Socket,
    buf: &'a mut [u8; 1024],
) -> Result<T, Box<dyn error::Error>> {
    let n = s.recv(buf)?;
    let msg = str::from_utf8(&buf[..n])?;
    serde_json::from_str::<T>(&msg).map_err(|e| e.into())
}
