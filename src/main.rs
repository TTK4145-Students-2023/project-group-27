use crossbeam_channel::select;
use std::time::*;

use driver_rust::elevio::elev;

pub mod doors;
pub mod inputs;
pub mod fsm;
pub mod requests;
pub mod config;

fn main() -> std::io::Result<()> {
    let elevator = elev::Elevator::init(config::ELEV_ADDR, config::ELEV_NUM_FLOORS)?;
    println!("Elevator started:\n{:#?}", elevator);

    // INITIALIZE INPUTS MODULE
    let poll_period = Duration::from_millis(25);
    let (
        call_button_rx, 
        floor_sensor_rx, 
        stop_button_rx, 
        obstruction_rx
    ) = inputs::init(elevator.clone(), poll_period);
    println!("module initialized: inputs");

    // INITIALIZE DOORS MODULE
    let (doors_activate_tx, doors_closing_rx) = doors::init(obstruction_rx.clone());
    println!("module initialized: doors");

    // INITIALIZE REQUESTS MODULE
    let (
        requests_should_stop_rx, 
        requests_next_direction_rx,
        requests_new_direction_tx
    ) = requests::init(
        elevator.clone(),
        call_button_rx.clone(),
        floor_sensor_rx.clone(),
    );
    println!("module initialized: requests");

    // INITIALIZE FSM MODULE
    fsm::init(
        elevator.clone(),
        requests_should_stop_rx.clone(), 
        doors_activate_tx.clone(),
        requests_next_direction_rx.clone(),
        doors_closing_rx.clone(),
        requests_new_direction_tx.clone()
    );
    println!("module initialized: fsm");

    // DRIVE ELEVATOR TO FLOOR IF INBETWEEN FLOORS
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(elev::DIRN_DOWN);
    }

    loop {
        select! {
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
