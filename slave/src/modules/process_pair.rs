use std::process;
use std::thread;
use std::time::Duration;
use std::process::Command;

use crossbeam_channel::{select, unbounded};
use network_rust::udpnet::{self, bcast::BcError};

use crate::utilities::elevator_status::ElevatorStatus;

pub fn process_pair(num_floors: u8, pp_update_port: u16) -> ElevatorStatus {
    println!("SLAVE process pair running on port: {:#?}\n------------------------", pp_update_port);
    let mut backup_data: ElevatorStatus = ElevatorStatus::new(num_floors);

    let (pp_update_tx, pp_update_rx) = unbounded::<ElevatorStatus>();
    thread::Builder::new().name("process_pair_recieve_from_slave".to_string()).spawn(move || {
        match udpnet::bcast::rx(pp_update_port, pp_update_tx) {
            Err(BcError::IOError(_e)) => process::exit(1),
            _ => (),
        }
    }).ok();

    loop {
        select! {
            recv(pp_update_rx) -> data => {
                backup_data = data.clone().unwrap();
            },
            default(Duration::from_secs(2)) => {
                return backup_data
            }
        }
    }
}

pub fn spawn_process_pair(
    program_path: String, 
    elevnum: u8,
    server_port: u16,
) {
    if cfg!(target_os = "linux") {
        Command::new("gnome-terminal")
            .arg("--")
            .arg("/bin/sh")
            .arg("-c")
            .arg("cd ".to_owned()
                + &program_path
                + " && "
                + "cargo run"
                + " --"
                + " --num "
                + &elevnum.to_string()
                + " --serverport "
                + &server_port.to_string())
            .output()
            .expect("failed to start backup");
    } else if cfg!(target_os = "macos") {
        Command::new("osascript")
            .arg("-e")
            .arg("tell app \"Terminal\" to do script \"cd ".to_owned() 
                + &program_path
                + " && "
                + "cargo run"
                + " --"
                + " --num "
                + &elevnum.to_string()
                + " --serverport "
                + &server_port.to_string() + "\"")
            .output()
            .expect("failed to start backup");
    }
}
