use std::thread;

use crossbeam_channel::{select, unbounded};

use crate::utils::debug::Debug;

pub mod doors;
pub mod io;
pub mod fsm;
pub mod network;
pub mod utils;

fn main() -> std::io::Result<()> {
    // READ CONFIGURATION
    let config = utils::config::Config::get();

    // INITIALIZE CHANNELS
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    let (master_hall_requests_tx, master_hall_requests_rx) = unbounded();
    let (elevator_behaviour_tx, elevator_behaviour_rx) = unbounded();

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
        config.settings.clone(),
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
        let elevator_settings = config.settings.clone();
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
            elevator_behaviour_tx,
        ));
    }

    // INITIALIZE NETWORK MODULE
    {
        let elevator_settings = config.settings.clone();
        let elevator_behaviour_rx = elevator_behaviour_rx.clone();
        thread::spawn(move || network::main(
            elevator_settings,
            config.network,
            hall_button_rx,
            master_hall_requests_tx,
            elevator_behaviour_rx,
        ));
    }

    let num_floors = config.settings.num_floors;
    let mut debug = Debug::new(num_floors);

    loop {
        select! {
            recv(elevator_behaviour_rx) -> msg => {
                debug.printstatus(&msg.unwrap()).unwrap();
            },
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
