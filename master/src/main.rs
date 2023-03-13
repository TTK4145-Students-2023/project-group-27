use std::thread;
use std::io::Result;

use crossbeam_channel::unbounded;

pub mod config;
pub mod network;
pub mod hall_request_assigner;
pub mod debug;

fn main() -> Result<()> {
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    thread::spawn(move || network::main(
        hall_requests_tx,
        connected_elevators_tx,
    ));

    thread::spawn(move || debug::main(
        hall_requests_rx,
        connected_elevators_rx,
    ));

    loop { }
}
