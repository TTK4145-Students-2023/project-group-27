use std::thread;

use crossbeam_channel::{select, unbounded};

use crate::utilities::debug::Debug;

mod doors;
mod io;
mod fsm;
mod network;

pub fn run() -> std::io::Result<()> {
    // READ CONFIGURATION
    let config = shared_resources::config::SlaveConfig::get();

    // INITIALIZE CHANNELS
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    let (master_hall_requests_tx, master_hall_requests_rx) = unbounded();
    let (elevator_status_tx, elevator_status_rx) = unbounded();

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
    );

    // INITIALIZE THREAD FOR DOOR EVENTS
    thread::spawn(move || doors::main(
        obstruction_rx,
        doors_activate_rx,
        doors_closing_tx,
        door_light_tx
    ));

    // INITIALIZE THREAD FOR STATE MACHINE
    {
        let elevator_settings = config.elevator.clone();
        thread::spawn(move || fsm::main(
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
        ));
    }

    // INITIALIZE NETWORK MODULE
    {
        let elevator_settings = config.elevator.clone();
        let elevator_status_rx = elevator_status_rx.clone();
        thread::spawn(move || network::main(
            elevator_settings,
            config.network,
            hall_button_rx,
            master_hall_requests_tx,
            elevator_status_rx,
        ));
    }

    let num_floors = config.elevator.num_floors;
    let mut debug = Debug::new(num_floors);

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
