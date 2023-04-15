use std::thread;
use std::io::Result;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::unbounded;
use shared_resources::config::MasterConfig;

mod network;
mod backup;

use crate::utilities::debug;

pub fn run() -> Result<()> {
    let config = MasterConfig::get();
    let num_floors = config.elevator.num_floors;
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (connected_elevators_tx, connected_elevators_rx) = unbounded();

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    println!("{:#?}", program_path);
    let backup_port = config.network.pp_port;
    //let ack_port = config.network.ack_port;
    let handle = thread::spawn(move || backup::backup(config.elevator.num_floors, backup_port));
    let backup_data = handle.join().unwrap();

    backup::spawn_backup(program_path);

    thread::spawn(move || network::main(
        backup_data,
        config,
        hall_requests_tx,
        connected_elevators_tx,
    ));
    
    thread::spawn(move || debug::main(
        num_floors,
        hall_requests_rx,
        connected_elevators_rx
    ));

    loop { }
}
