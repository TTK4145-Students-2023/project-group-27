use std::thread;
use std::process;
use std::process::Command;
use std::time::Duration;

use crossbeam_channel::{select, unbounded};

use shared_resources::call::Call;
use network_rust::udpnet;
use network_rust::udpnet::bcast::BcError;

pub fn process_pair(num_floors: u8, process_pair_port: u16) -> Vec<Vec<bool>> {
    println!("BACKUP process pair running on port: {:#?}\n---------------------", process_pair_port);

    let (process_pair_tx, process_pair_rx) = unbounded::<Vec<Vec<bool>>>();
    thread::Builder::new().name("process_pair_recieve_from_master".to_string()).spawn(move || {
        match udpnet::bcast::rx(process_pair_port, process_pair_tx) {
            Err(BcError::IOError(_e)) => process::exit(1),
            _ => (),
        }
    }).ok();

    let mut backup_data = vec![vec![false; Call::num_hall_calls() as usize]; num_floors as usize];

    loop {
        select! {
            recv(process_pair_rx) -> data => {
                backup_data = data.unwrap_or(backup_data);
            },
            default(Duration::from_secs(2)) => {
                return backup_data;
            }
        }
    }
}

pub fn spawn_process_pair(
    program_path: String
) {
    if cfg!(target_os = "linux") {
        Command::new("gnome-terminal")
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg("cd ".to_owned()
                + &program_path
                + " && "
                + "cargo run")
            .output()
            .expect("failed to spawn process pair");
    } else if cfg!(target_os = "macos") {
        Command::new("osascript")
            .arg("-e")
            .arg("tell app \"Terminal\" to do script \"cd ".to_owned() 
                + &program_path
                + " && "
                + "cargo run\"")
            .output()
            .expect("failed to spawn process pair");
    }
}
