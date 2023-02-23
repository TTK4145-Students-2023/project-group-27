use std::time::Duration;

use crossbeam_channel::select;
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
    let (doors_activate_tx, doors_closing_rx) = doors::init(obstruction_rx);
    println!("module initialized: doors");

    // INITIALIZE REQUESTS MODULE
    let (
        requests_should_stop_rx, 
        requests_next_direction_rx
    ) = requests::init(
        elevator.clone(),
        call_button_rx,
        floor_sensor_rx,
    );
    println!("module initialized: requests");

    // INITIALIZE FSM MODULE
    fsm::init(
        elevator.clone(),
        requests_should_stop_rx, 
        doors_activate_tx,
        requests_next_direction_rx,
        doors_closing_rx
    );
    println!("module initialized: fsm");

    loop {
        select! {
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
