use crossbeam_channel::select;
use std::time::*;

use driver_rust::elevio::elev;

pub mod doors;
pub mod inputs;
pub mod fsm;
pub mod requests;

const ELEV_NUM_FLOORS: u8 = 4;
const ELEV_ADDR: &str = "localhost:15657";

fn main() -> std::io::Result<()> {
    let (doors_activate_tx, doors_closing_rx) = doors::init();

    let elevator = elev::Elevator::init(ELEV_ADDR, ELEV_NUM_FLOORS)?;
    println!("Elevator started:\n{:#?}", elevator);

    let poll_period = Duration::from_millis(25);

    let (
        call_button_rx, 
        floor_sensor_rx, 
        stop_button_rx, 
        obstruction_rx
    ) = inputs::init(elevator.clone(), poll_period);

    let mut dirn = elev::DIRN_DOWN;
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(dirn);
    }

    fsm::main(
        elevator.clone(), 
        call_button_rx,
        floor_sensor_rx,
        doors_closing_rx,
        obstruction_rx,
        doors_activate_tx
    );

    loop {

    }
}

