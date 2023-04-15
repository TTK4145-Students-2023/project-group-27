use std::thread;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

use crossbeam_channel::{select, unbounded};

use crate::utilities::debug::Debug;

mod doors;
mod io;
mod fsm;
mod network;
mod backup;

pub fn run() -> std::io::Result<()> {    
    // READ CONFIGURATION
    let config = shared_resources::config::SlaveConfig::get();
    println!("elevnum: {}, serverport: {}", config.elevnum, config.server.port);

    let program_dir = PathBuf::from("./.");
    let program_path: String = fs::canonicalize(&program_dir).unwrap().into_os_string().into_string().unwrap();
    let pp_update_port = config.network.pp_update_port;
    let pp_ack_port = config.network.pp_ack_port;
    let handle = thread::spawn(move || backup::backup(config.elevator.num_floors, pp_update_port, pp_ack_port));
    let backup_data = handle.join().unwrap();
   
    // BECOME MAIN, CREATE NEW BACKUP
    backup::spawn_backup(program_path, config.elevnum, config.server.port);

    // INITIALIZE CHANNELS
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    let (master_hall_requests_tx, master_hall_requests_rx) = unbounded();
    let (elevator_status_tx, elevator_status_rx) = unbounded();

    // INITIALIZE INPUTS MODULE
    let config1 = config.clone();
    let (
        cab_button_rx, 
        hall_button_rx, 
        floor_sensor_rx, 
        stop_button_rx, 
        obstruction_rx,
        stop_button_light_tx,
        button_light_tx,
        motor_direction_tx,
        door_light_tx,
        floor_indicator_tx,
    ) = io::init(
        config1.server,
        config1.elevator.clone(),
    )?;


    // INITIALIZE THREAD FOR DOOR EVENTS
    thread::Builder::new().name("doors".to_string()).spawn(move || doors::main(
        obstruction_rx,
        doors_activate_rx,
        doors_closing_tx,
        door_light_tx
    ))?;

    // INITIALIZE THREAD FOR STATE MACHINE
    thread::Builder::new().name("fsm".to_string()).spawn(move || fsm::main(
        backup_data,
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

    // INITIALIZE NETWORK MODULE
    {
        let config = config.clone();
        let elevator_status_rx = elevator_status_rx.clone();
        thread::Builder::new().name("network".to_string()).spawn(move || network::main(
            config,
            hall_button_rx,
            master_hall_requests_tx,
            elevator_status_rx,
        ))?;
    }

    let num_floors = config.elevator.num_floors;
    let mut debug = Debug::new(num_floors);
    let mut packetloss_active = false;
    
    loop {
        select! {
            recv(elevator_status_rx) -> msg => {
                debug.printstatus(&msg.unwrap()).unwrap();
            },
            recv(stop_button_rx) -> msg => {
                // apply 25% packet loss on master communication ports if in debug mode
                if msg.unwrap() && cfg!(debug_assertions) {
                    let exec_path = "packetloss";
                    let command = match packetloss_active {
                        false => "sudo ".to_owned() 
                                + exec_path 
                                + " -p " + &config.network.command_port.to_string()
                                + "," + &config.network.update_port.to_string() 
                                + " -r 0.25",
                        true => "sudo ".to_owned() 
                                + exec_path 
                                + " -f",
                    };
                    Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .output()
                        .expect("failed to induce packetloss ");
                    packetloss_active = !packetloss_active;
                    stop_button_light_tx.send(packetloss_active).unwrap();
                }
            }
        }
    }
}
