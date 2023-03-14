use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender};
use driver_rust::elevio::elev::{self, DIRN_DOWN};

#[derive(PartialEq, Debug)]
enum State {
    Idle,
    Moving,
    DoorOpen,
}

pub fn main(
    should_stop_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>,
    next_direction_rx: Receiver<u8>,
    doors_closing_rx: Receiver<bool>,
    motor_direction_tx: Sender<u8>,
    floor_sensor_rx: Receiver<u8>,
    floor_indicator_tx: Sender<u8>,
    elevator_state_tx: Sender<(String,u8,u8)>,
    elevator_data_tx: Sender<(u8,u8,bool)>,
) {
    const UPDATE_FREQ: f64 = 0.25;

    let mut floor: u8 = 0;
    let mut direction: u8 = DIRN_DOWN;
    let mut state: State = State::Moving;

    loop {
        select! {
            recv(floor_sensor_rx) -> msg => {
                floor = msg.unwrap();
                floor_indicator_tx.send(floor).unwrap();
                let is_stopped = state != State::Moving;
                elevator_data_tx.send((floor, direction, is_stopped)).unwrap();
            },
            recv(should_stop_rx) -> _ => {
                match state {
                    State::Idle => {
                        state = State::DoorOpen;
                        doors_activate_tx.send(true).unwrap();
                    },
                    State::Moving => {
                        state = State::DoorOpen;
                        motor_direction_tx.send(elev::DIRN_STOP).unwrap();
                        doors_activate_tx.send(true).unwrap();
                    },
                    State::DoorOpen => (),
                }
            },
            recv(next_direction_rx) -> dirn => {
                match state {
                    State::Idle => {
                        match dirn.unwrap() {
                            elev::DIRN_UP | elev::DIRN_DOWN => {
                                motor_direction_tx.send(dirn.unwrap()).unwrap();
                                direction = dirn.unwrap();
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
                        state = State::Idle;
                    },
                }
            },
            default(Duration::from_secs_f64(UPDATE_FREQ)) => {
                if state != State::Moving { // TODO: consider negating this logic
                    elevator_data_tx.send((floor, direction, true)).unwrap();
                }
            },
        }

        // for every loop: send state information to network module
        let state_str = match state {
            State::Idle => "idle",
            State::Moving => "moving",
            State::DoorOpen => "doorOpen",
        };
        elevator_state_tx.send((
            String::from(state_str),
            floor,
            direction,
        )).unwrap();
    }
}
