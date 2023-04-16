use std::thread;
use std::io::Result;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use std::process;

use crossbeam_channel::{unbounded, select};

use shared_resources::config::BackupConfig;
use shared_resources::request::Request;
use network_rust::udpnet;

mod process_pair;

pub fn main() -> Result<()> {
    let config = BackupConfig::get();
    let num_floors = config.elevator.num_floors;
    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    let process_pair_port = config.network.pp_port;
    let process_pair_handle = thread::spawn(move || process_pair::process_pair(num_floors, process_pair_port));
    let mut backup_data = process_pair_handle.join().unwrap();

    process_pair::spawn_process_pair(program_path);

    let (changed_requests_tx, changed_requests_rx) = unbounded::<(Vec<Request>,Vec<Request>)>();
    thread::Builder::new().name("master_to_backup".to_string()).spawn(move || {
        if udpnet::bcast::rx(config.network.backup_update_port, changed_requests_tx).is_err() {
            process::exit(1);
        }
    }).unwrap();

    let (broadcast_backup_data_tx, broadcast_backup_data_rx) = unbounded::<Vec<Vec<bool>>>();
    let broadcast_backup_data_rx1 = broadcast_backup_data_rx.clone();
    thread::Builder::new().name("backup_to_master".to_string()).spawn(move || {
        if udpnet::bcast::tx(config.network.backup_ack_port, broadcast_backup_data_rx1, false).is_err() {
            process::exit(1);
        }
    }).unwrap();
    thread::Builder::new().name("backup_to_process_pair".to_string()).spawn(move || {
        if udpnet::bcast::tx(process_pair_port, broadcast_backup_data_rx, true).is_err() {
            process::exit(1);
        }
    }).unwrap();

    println!("Backup is running...");
    loop {
        select!{
            recv(changed_requests_rx) -> msg => {
                let changed_requests = msg.unwrap();

                for new_request in changed_requests.0 {
                    backup_data[new_request.floor as usize][new_request.call as usize] = true;
                }
                for served_request in changed_requests.1 {
                    backup_data[served_request.floor as usize][served_request.call as usize] = false;
                }

                broadcast_backup_data_tx.send(backup_data.clone()).unwrap();
            },
            default(Duration::from_secs_f64(0.1)) => {
                broadcast_backup_data_tx.send(backup_data.clone()).unwrap();
            }
        }
    }
}
