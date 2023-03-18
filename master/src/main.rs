use std::thread;
use std::io::Result;

use crossbeam_channel::unbounded;
use shared_resources::config::MasterConfig;

use master::network;
use master::utilities::debug;

fn main() -> Result<()> {
    let config = MasterConfig::get();
    let num_floors = config.elevator.num_floors;
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    thread::spawn(move || network::main(
        config,
        hall_requests_tx,
        connected_elevators_tx,
    ));

    thread::spawn(move || debug::main(
        num_floors,
        hall_requests_rx,
        connected_elevators_rx,
    ));

    loop { }
}
