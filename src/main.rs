use crossbeam_channel::{unbounded, select};
use std::thread::*;
use std::time::*;

use driver_rust::elevio::elev;

pub mod doors;
pub mod inputs;

const ELEV_NUM_FLOORS: u8 = 4;
const ELEV_ADDR: &str = "localhost:15657";

fn main() -> std::io::Result<()> {
    let (sthread, rmain) = unbounded();
    let (smain, rthread) = unbounded();

    spawn(move || doors::main(sthread, rthread));

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

    loop {
        select! {
            recv(call_button_rx) -> a => {
                let call_button = a.unwrap();
                println!("{:#?}", call_button);
                elevator.call_button_light(call_button.floor, call_button.call, true);
            },
            recv(floor_sensor_rx) -> a => {
                let floor = a.unwrap();
                println!("Floor: {:#?}", floor);
                dirn =
                    if floor == 0 {
                        elev::DIRN_UP
                    } else if floor == ELEV_NUM_FLOORS-1 {
                        elev::DIRN_DOWN
                    } else {
                        dirn
                    };
                elevator.motor_direction(dirn);
            },
            recv(stop_button_rx) -> a => {
                let stop = a.unwrap();
                println!("Stop button: {:#?}", stop);
                for f in 0..ELEV_NUM_FLOORS {
                    for c in 0..3 {
                        elevator.call_button_light(f, c, false);
                    }
                }
            },
            recv(obstruction_rx) -> a => {
                let obstr = a.unwrap();
                println!("Obstruction: {:#?}", obstr);
                elevator.motor_direction(if obstr { elev::DIRN_STOP } else { dirn });
            },
        }
    }
}

