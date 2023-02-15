use crossbeam_channel::select;
use std::time::*;


use driver_rust::elevio::elev;

pub mod doors;
pub mod inputs;
pub mod orders;

const ELEV_NUM_FLOORS: u8 = 4;
const ELEV_ADDR: &str = "localhost:15657";

fn main() -> std::io::Result<()> {
    let (doors_activate_tx, doors_timed_out_rx) = doors::init();

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
                // STOP AND OPEN DOOR
                elevator.motor_direction(elev::DIRN_STOP);
                doors_activate_tx.send(true).unwrap();
                elevator.door_light(true);
                doors_timed_out_rx.recv().unwrap();
                elevator.door_light(false);
                // CONTINUE...
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
            }
        }
    }
}

