use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, select, tick};
use driver_rust::elevio::elev::{elev, poll, self, DIRN_DOWN};

use crate::config::ElevatorSettings;

#[derive(PartialEq, Debug)]
enum State {
    Idle,
    Moving,
    DoorOpen,
}

#[derive(Clone, Debug)]
pub struct ElevatorStatus {
    pub orders: Vec<Vec<bool>>,
    pub cleared_request: poll::CallButton,
    pub state: String,
    pub floor: u8,
    pub direction: u8
}

#[derive(Clone, Debug)]
pub struct HallRequests {
    pub our_requests: Vec<[bool; 2]>,
    pub all_requests: Vec<[bool; 2]>
}



pub fn main(
    elevator_settings: ElevatorSettings,
    cab_button_rx: Receiver<poll::CallButton>, // From io
    //our_hall_requests_rx: Receiver<Vec<[bool; 2]>>, // From network
    //all_hall_requests_rx: Receiver<Vec<[bool; 2]>>, // From network
    hall_requests_rx: Receiver<HallRequests>, //From network
    doors_closing_rx: Receiver<bool>, // From doors
    floor_sensor_rx: Receiver<u8>, // From io
    cab_requests_tx: Sender<Vec<bool>>, // To network
    button_light_tx: Sender<(u8,u8,bool)>, // To io
    doors_activate_tx: Sender<bool>, // To doors
    motor_direction_tx: Sender<u8>, // To io
    floor_indicator_tx: Sender<u8>, // To io
    // orders_tx: Sender<Vec<Vec<bool>>>, // To debug
    // cleared_request_tx: Sender<poll::CallButton>, // To network
    // elevator_state_tx: Sender<(String,u8,u8)>, // To network
    elevator_status_tx: Sender<ElevatorStatus> // To network & debug
) {

    let timer = tick(Duration::from_secs_f64(0.1));
    let n_buttons = elevator_settings.num_buttons;
    let n_floors = elevator_settings.num_floors;

    let mut floor: u8 = 0;
    let mut direction: u8 = DIRN_DOWN;
    let mut state: State = State::Moving;
    let mut orders = vec![vec![false; n_buttons as usize]; n_floors as usize];

    // CLEAR ALL LIGHTS
    for floor in 0..n_floors {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap();
        }
    }

    loop {
        select! {
            recv(cab_button_rx) -> msg => {
                State::Idle => {
                    state = State:DoorOpen;
                    doors_activate_tx.send(true).unwrap();
                },
                State::Moving => {
                    let destination = msg.clone().unwrap().floor;
                    // --- Consider turning this part into a function
                    orders[destination as usize][elev::CAB as usize] = true;
                    button_light_tx.send((destination, elev::CAB, true)).unwrap();
                    let mut cab_requests = vec![false; n_floors as usize];
                    for floor in 0..n_floors {
                        cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                    }
                    cab_requests_tx.send(cab_requests).unwrap();
                    // This cab order vector can be unpacked at network instead of here
                    // --- or a method to a struct
                },
                State::DoorOpen => {
                    doors_activate_tx.send(true).unwrap();
                },
                
            },
            recv(hall_requests_rx) -> msg => {
                // --- Consider turning this part into a function
                State::Idle => {
                    for f in 0..n_floors {
                        for b in elev::HALL_UP..=elev::HALL_DOWN {
                            if msg.clone().unwrap().all_hall_requests[f as usize][b as usize] {
                                /* Keep door open if elevator gets an order it already is at without sending further. */
                            }
                            else {
                                button_light_tx.send((f, b, msg.clone().unwrap().all_hall_requests[f as usize][b as usize])).unwrap();
                                orders[f as usize][b as usize] = msg.clone().unwrap().our_hall_requests[f as usize][b as usize];
                            }
                            
                        }
                    }
                },
                State::Moving => {

                },
                State::DoorOpen => {

                }
                // --- or a method to the struct
            },
            recv() -> _ => {

            },
            recv() -> _ => {

            },
            recv(timer) -> _ => {
                /* Currently, nothing needs to be sent with timer. */
            }
        }

        // This part is run to get the latest update of states, no matter which recv() was used.
        let state_str = match state {
            State::Idle => "idle",
            State::Moving => "moving",
            State::DoorOpen => "doorOpen",
        }; // State is converted to string such that we don't need the state enum dependency at "network"
        // And also, the string is needed for the hall request assigner. 

        elevator_status_tx.send(ElevatorStatus {
            orders: orders,
            cleared_request: poll::CallButton {
                floor: floor, // This one is from former "elevator_data_rx" in "requests"
                call: if direction == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN },
            },
            state: String::from(state_str),
            floor: floor, // This one is presumed to be the same as the one above
            direction,
        }).unwrap();
    }
}