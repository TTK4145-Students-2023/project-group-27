use std::thread::spawn;

use crossbeam_channel::{select, Receiver, Sender};
use driver_rust::elevio::elev::{self, Elevator};

#[derive(PartialEq, Debug)]
enum State {
    Idle,
    Moving,
    DoorOpen
}

pub fn init(
    elevator: Elevator,
    requests_should_stop_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>,
    requests_next_direction_rx: Receiver<u8>,
    doors_closing_rx: Receiver<bool>,
) {
    spawn(move || main(
        elevator, 
        requests_should_stop_rx, 
        doors_activate_tx, 
        requests_next_direction_rx,
        doors_closing_rx,
    ));
}

fn main(
    elevator: Elevator,
    requests_should_stop_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>,
    requests_next_direction_rx: Receiver<u8>,
    doors_closing_rx: Receiver<bool>,
) {
    let mut state: State = State::Idle;
    println!("started state machine in state: {:#?}", state);

    // DRIVE ELEVATOR TO FLOOR
    if elevator.floor_sensor().is_none() {
        elevator.motor_direction(elev::DIRN_DOWN);
        state = State::Moving;
        println!("finding floor in state: {:#?}", state);
    }

    loop {
        select! {
            recv(requests_should_stop_rx) -> _ => {
                match state {
                    State::Idle => {
                        state = State::DoorOpen;
                        doors_activate_tx.send(true).unwrap();
                        elevator.door_light(true);
                    },
                    State::Moving => {
                        state = State::DoorOpen;
                        elevator.motor_direction(elev::DIRN_STOP);
                        doors_activate_tx.send(true).unwrap();
                        elevator.door_light(true);
                    },
                    State::DoorOpen => (),
                }
            },
            recv(requests_next_direction_rx) -> dirn => {
                match state {
                    State::Idle => {
                        match dirn.unwrap() {
                            elev::DIRN_UP | elev::DIRN_DOWN => {
                                elevator.motor_direction(dirn.unwrap());
                                state = State::Moving;
                            },
                            _ => ()
                        }
                    },
                    State::Moving => (),
                    State::DoorOpen => (),
                }
            },
            recv(doors_closing_rx) -> _ => {
                match state {
                    State::Idle => (),
                    State::Moving => (),
                    State::DoorOpen => {
                        elevator.door_light(false);
                        state = State::Idle;
                    },
                }
            }
        }
    }
}
