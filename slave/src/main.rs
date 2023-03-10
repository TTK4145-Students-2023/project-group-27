use std::thread;

use crossbeam_channel::{select, unbounded};

pub mod doors;
pub mod io;
pub mod fsm;
pub mod requests;
pub mod config;
pub mod network;

fn main() -> std::io::Result<()> {
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();

    let (should_stop_tx, should_stop_rx) = unbounded();
    let (next_direction_tx, next_direction_rx) = unbounded();
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (cab_requests_tx, cab_requests_rx) = unbounded();
    let (elevator_data_tx, elevator_data_rx) = unbounded();

    let (elevator_state_tx, elevator_state_rx) = unbounded();

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
    ) = io::init();
    println!("module initialized: inputs");

    // INITIALIZE DOORS MODULE
    thread::spawn(move || doors::main(
        doors_closing_tx, 
        doors_activate_rx, 
        obstruction_rx,
        door_light_tx
    ));
    println!("module initialized: doors");

    // INITIALIZE REQUESTS MODULE
    thread::spawn(move || requests::main(
        cab_button_rx, 
        hall_requests_rx,
        button_light_tx,
        should_stop_tx,
        next_direction_tx,
        cab_requests_tx,
        elevator_data_rx
    ));
    println!("module initialized: requests");

    // INITIALIZE FSM MODULE
    thread::spawn(move || fsm::main(
        should_stop_rx, 
        doors_activate_tx, 
        next_direction_rx,
        doors_closing_rx,
        motor_direction_tx,
        floor_sensor_rx,
        floor_indicator_tx,
        elevator_state_tx,
        elevator_data_tx,
    ));
    println!("module initialized: fsm");

    // INITIALIZE NETWORK MODULE
    thread::spawn(move || network::main(
        hall_button_rx, 
        hall_requests_tx, 
        elevator_state_rx,
        cab_requests_rx,
    ));
    println!("module initialized: network");

    loop {
        select! {
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
