use std::io;
use std::net;

use socket2::{Domain, Protocol, SockAddr, Socket, Type};

pub fn new_tx(port: u16) -> io::Result<(Socket, SockAddr)> {
    let sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?;
    let remote_addr = net::SocketAddr::from(([255, 255, 255, 255], port));
    Ok((sock, remote_addr.into()))
}

pub fn new_rx(port: u16) -> io::Result<Socket> {
    let sock = Socket::new(Domain::ipv4(), Type::dgram(), Some(Protocol::udp()))?;
    sock.set_broadcast(true)?;
    sock.set_reuse_address(true)?;
    let local_addr = net::SocketAddr::from(([0, 0, 0, 0], port));
    sock.bind(&local_addr.into())?;
    Ok(sock)
}
