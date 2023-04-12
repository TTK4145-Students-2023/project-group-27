use std::thread;
use std::process;
use std::process::Command;
use std::time::Duration;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::{select, unbounded};
use network_rust::udpnet;

use crate::utilities::debug::Debug;
use crate::utilities::elevator_status::ElevatorStatus;

mod doors;
mod io;
mod fsm;
mod network;

fn backup(num_floors: u8, backup_port: u16) -> ElevatorStatus {
    println!("BACKUP MODE\n------------");
    let mut backup_data: ElevatorStatus = ElevatorStatus::new(num_floors);

    let (backup_recv_tx, backup_recv_rx) = unbounded::<ElevatorStatus>();
    thread::Builder::new().name("backup_recieve_from_elevator".to_string()).spawn(move || {
        //panic::set_hook(Box::new(|_| {println!("Went from backup to master")}));
        if udpnet::bcast::rx(backup_port, backup_recv_tx).is_err() {
            process::exit(1);
        }
        //let _ = panic::take_hook();
    }).ok();

    //let config = shared_resources::config::SlaveConfig::get();
    //let num_floors = config.elevator.num_floors;
    //let mut backup_debug = Debug::new(num_floors);

    loop {
        select! {
            recv(backup_recv_rx) -> data => {
                //backup_debug.printstatus(&data.clone().unwrap()).unwrap();
                backup_data = data.unwrap();
            },
            default(Duration::from_secs(2)) => {
                break;
            }
        }
    }
    backup_data
}

pub fn run() -> std::io::Result<()> {    
    // READ CONFIGURATION
    let config = shared_resources::config::SlaveConfig::get();

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    println!("{:#?}",program_path);
    let backup_port = config.network.backup_port; // Magic nuber
    let handle = thread::spawn(move || backup(config.elevator.num_floors, backup_port));
    let backup_data = handle.join().unwrap();
    // BECOME MAIN, CREATE NEW BACKUP

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
                .expect("failed to start backup");
    } else if cfg!(target_os = "macOS") {
        Command::new("osascript")
                .arg("-e")
                .arg("tell app \"Terminal\" to do script \"cd ".to_owned() 
                + program_path.as_str() 
                + "; cargo run -- " 
                + program_path.as_str() + "\"")
                .output()
                .expect("failed to start backup");
    }

    // INITIALIZE CHANNELS
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    let (master_hall_requests_tx, master_hall_requests_rx) = unbounded();
    let (elevator_status_tx, elevator_status_rx) = unbounded();
    let (backup_send_tx, backup_send_rx) = unbounded::<ElevatorStatus>();

    // INITIALIZE INPUTS MODULE
    let (
        cab_button_rx, 
        hall_button_rx, 
        floor_sensor_rx, 
        stop_button_rx, 
        obstruction_rx,
        button_light_tx,
        motor_direction_tx,
        door_light_tx,
        floor_indicator_tx,
    ) = io::init(
        config.server,
        config.elevator.clone(),
    )?;
    
    // INITIALIZE THREAD FOR DOOR EVENTS
    thread::Builder::new().name("doors".to_string()).spawn(move || doors::main(
        obstruction_rx,
        doors_activate_rx,
        doors_closing_tx,
        door_light_tx
    ))?;

    // INITIALIZE THREAD FOR STATE MACHINE
    {
        let elevator_settings = config.elevator.clone();
        thread::Builder::new().name("fsm".to_string()).spawn(move || fsm::main(
            backup_data,
            backup_send_tx,
            elevator_settings,
            floor_sensor_rx,
            floor_indicator_tx,
            button_light_tx,
            doors_closing_rx,
            doors_activate_tx,
            cab_button_rx,
            motor_direction_tx,
            master_hall_requests_rx,
            elevator_status_tx,
        ))?;
    }

    // INITIALIZE NETWORK MODULE
    {
        let elevator_settings = config.elevator.clone();
        let elevator_status_rx = elevator_status_rx.clone();
        thread::Builder::new().name("network".to_string()).spawn(move || network::main(
            elevator_settings,
            config.network,
            hall_button_rx,
            master_hall_requests_tx,
            elevator_status_rx,
        ))?;
    }

    let num_floors = config.elevator.num_floors;
    let mut debug = Debug::new(num_floors);

    {
        thread::Builder::new().name("backup_udp_sender".to_string()).spawn(move || {
            if udpnet::bcast::tx(backup_port, backup_send_rx).is_err() {
                // crash program if creating the socket fails (`bcast:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        })?;
    }

    loop {
        select! {
            recv(elevator_status_rx) -> msg => {
                debug.printstatus(&msg.unwrap()).unwrap();
            },
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
