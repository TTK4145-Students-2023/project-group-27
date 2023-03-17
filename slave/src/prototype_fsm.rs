use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender, tick};
use driver_rust::elevio::{elev, poll};

use crate::config::ElevatorSettings;

#[derive(PartialEq, Debug)]
enum Behavior {
    Idle,
    Moving,
    DoorOpen,
}

#[derive(Clone, Debug)] // As for now, the CallButton does not have clone trait, so use as_ref() as for now.
pub struct ElevatorStatus {
    pub orders: Vec<Vec<bool>>,
    pub behavior: String,
    pub floor: u8,
    pub direction: u8
}

#[derive(Clone, Debug)]
pub struct HallRequests {
    pub our_requests: Vec<[bool; 2]>,
    pub all_requests: Vec<[bool; 2]>
}

// --- Functions ---
fn should_stop(
    n_floors: u8,
    orders: &Vec<Vec<bool>>,
    floor: u8,
    dirn: u8
) -> bool {
    if cab_request_at_floor(&orders, floor)
    || requests_in_direction_at_this_floor(&orders, floor, dirn)
    || !further_requests_in_direction(n_floors, &orders, floor, dirn) {
        return true
    }
    false
}

fn cab_request_at_floor(
    orders: &Vec<Vec<bool>>,
    floor: u8
) -> bool {
    orders[floor as usize][elev::CAB as usize]
}

fn requests_in_direction_at_this_floor(
    orders: &Vec<Vec<bool>>,
    floor: u8,
    dirn: u8,
) -> bool {
    let hall_button = if dirn == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN };
    orders[floor as usize][hall_button as usize]
}

fn further_requests_in_direction(
    n_floors: u8,
    orders: &Vec<Vec<bool>>,
    floor: u8,
    dirn: u8,
) -> bool {
    let range = if dirn == elev::DIRN_UP { (floor+1)..n_floors } else { 0..floor };
    for f in range {
        for btn in elev::HALL_UP..=elev::CAB {
            if orders[f as usize][btn as usize] {
                return true
            }
        }
    }
    false
}

fn next_direction(
    n_floors: u8,
    orders: &Vec<Vec<bool>>,
    floor: u8,
    last_direction: u8
) -> u8 {
    let other_direction = if last_direction == elev::DIRN_UP { elev::DIRN_DOWN } else { elev::DIRN_UP };
    if further_requests_in_direction(n_floors, &orders, floor, last_direction) {
        return last_direction
    } else if further_requests_in_direction(n_floors, &orders, floor, other_direction) {
        return other_direction
    }
    elev::DIRN_STOP
}

fn update_orders(
    n_floors: u8,
    n_buttons: u8,
    hall_requests: HallRequests,
    button_light_tx: Sender<(u8,u8,bool)>
) -> Vec<Vec<bool>> {
    let mut orders = vec![vec![false; n_buttons as usize]; n_floors as usize];
    let our_requests = hall_requests.our_requests;
    let all_requests = hall_requests.all_requests;

    for f in 0..n_floors {
        for b in elev::HALL_UP..=elev::HALL_DOWN {
            button_light_tx.send((f, b, all_requests[f as usize][b as usize])).unwrap();
            orders[f as usize][b as usize] = our_requests[f as usize][b as usize];
        }
    }
    orders
}

// --- Functions ---

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
    cleared_request_tx: Sender<poll::CallButton>, // To network
    // orders_tx: Sender<Vec<Vec<bool>>>, // To debug
    // elevator_state_tx: Sender<(String,u8,u8)>, // To network
    elevator_status_tx: Sender<ElevatorStatus> // To network & debug
) {

    let timer = tick(Duration::from_secs_f64(0.1));
    let n_buttons = elevator_settings.num_buttons;
    let n_floors = elevator_settings.num_floors;

    let mut floor: u8 = 0;
    let mut direction: u8 = elev::DIRN_DOWN;
    let mut behavior: Behavior = Behavior::Moving;
    let mut orders = vec![vec![false; n_buttons as usize]; n_floors as usize];
    let mut cab_requests = vec![false; n_floors as usize];

    // CLEAR ALL LIGHTS
    for floor in 0..n_floors {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap();
        }
    }

    loop {
        select! {
            recv(cab_button_rx) -> msg => {
                match behavior {
                    Behavior::Idle => {
                        if floor == msg.as_ref().unwrap().floor {
                            behavior = Behavior::DoorOpen;
                            doors_activate_tx.send(true).unwrap();
                        }
                        else {
                            let destination = msg.as_ref().unwrap().floor;
                            // --- Consider turning this part into a function
                            orders[destination as usize][elev::CAB as usize] = true;
                            button_light_tx.send((destination, elev::CAB, true)).unwrap();
                            for floor in 0..n_floors {
                                cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                            }
                            cab_requests_tx.send(cab_requests.clone()).unwrap();
                        }
                    },
                    Behavior::Moving => {
                        let destination = msg.as_ref().unwrap().floor;
                        // --- Consider turning this part into a function
                        orders[destination as usize][elev::CAB as usize] = true;
                        button_light_tx.send((destination, elev::CAB, true)).unwrap();
                        for floor in 0..n_floors {
                            cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                        }
                        cab_requests_tx.send(cab_requests.clone()).unwrap();
                        // This cab order vector can be unpacked at network instead of here
                        // --- or a method to a struct
                    },
                    Behavior::DoorOpen => {
                        if floor == msg.as_ref().unwrap().floor {
                            doors_activate_tx.send(true).unwrap();
                        }
                        else {
                            let destination = msg.as_ref().unwrap().floor;
                            // --- Consider turning this part into a function
                            orders[destination as usize][elev::CAB as usize] = true;
                            button_light_tx.send((destination, elev::CAB, true)).unwrap();
                            for floor in 0..n_floors {
                                cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
                            }
                            cab_requests_tx.send(cab_requests.clone()).unwrap();
                        }
                    },
                }
            },
            recv(hall_requests_rx) -> msg => { // Not prioritized, since this is for debug
                // --- Consider turning this part into a function
                let all_requests = msg.clone().unwrap().all_requests;
                let our_requests = msg.clone().unwrap().our_requests;
                match behavior {
                    Behavior::Moving => {
                        //orders = update_orders(n_floors, n_buttons, msg.clone().unwrap(), button_light_tx.clone());
                        for f in 0..n_floors {
                            for b in elev::HALL_UP..=elev::HALL_DOWN {
                                button_light_tx.send((f, b, all_requests[f as usize][b as usize])).unwrap();
                                orders[f as usize][b as usize] = our_requests[f as usize][b as usize];
                            }
                        }
                    },
                    _ => {
                        for f in 0..n_floors {
                            for b in elev::HALL_UP..=elev::HALL_DOWN {
                                if our_requests[f as usize][b as usize] && f == floor {
                                    if !further_requests_in_direction(n_floors, &orders, floor, direction)
                                    || cab_requests[f as usize] == orders[f as usize][b as usize] {
                                        behavior = Behavior::DoorOpen;
                                        doors_activate_tx.send(true).unwrap();
                                        button_light_tx.send((floor, elev::CAB, false)).unwrap();
                                        orders[f as usize][b as usize] = false;
                                        cleared_request_tx.send(poll::CallButton {
                                        floor: f,
                                        call: if direction == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN },
                                    }).unwrap();
                                        cleared_request_tx.send(poll::CallButton {
                                            floor: floor,
                                            call: if direction == elev::DIRN_UP { elev::HALL_DOWN } else { elev::HALL_UP },
                                        }).unwrap();
                                    }
                                    /* Keep door open if elevator gets an order it already is at without sending further. */
                                }
                                else {
                                    button_light_tx.send((f, b, all_requests[f as usize][b as usize])).unwrap();
                                    orders[f as usize][b as usize] = our_requests[f as usize][b as usize];
                                }
                            }
                        }
                    },
                }
                // --- or a method to the struct
            },
            recv(floor_sensor_rx) -> msg => {
                floor = msg.unwrap();
                floor_indicator_tx.send(floor).unwrap();
                match behavior {
                    Behavior::Moving => {
                        if should_stop(n_floors, &orders, floor, direction) {
                            behavior = Behavior::DoorOpen;
                            motor_direction_tx.send(elev::DIRN_STOP).unwrap();
                            doors_activate_tx.send(true).unwrap();

                            orders[floor as usize][elev::CAB as usize] = false;
                            button_light_tx.send((floor, elev::CAB, false)).unwrap();
                            cleared_request_tx.send(poll::CallButton {
                                floor: floor,
                                call: if direction == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN },
                            }).unwrap();
                            if !further_requests_in_direction(n_floors, &orders, floor, direction) {
                                cleared_request_tx.send(poll::CallButton {
                                    floor: floor,
                                    call: if direction == elev::DIRN_UP { elev::HALL_DOWN } else { elev::HALL_UP },
                                }).unwrap();
                            }
                        }
                    },
                    _ => (), // This case should not be possible as a still elevator does not reach a new floor
                }
            },
            recv(doors_closing_rx) -> _ => {
                match behavior {
                    Behavior::DoorOpen => {
                        behavior = Behavior::Idle;
                    },
                    _ => (),
                }
            },
            recv(timer) -> _ => {
                match behavior {
                    Behavior::Idle => {
                        let next_direction = next_direction(n_floors, &orders, floor, direction);
                        match next_direction {
                            elev::DIRN_UP | elev::DIRN_DOWN => {
                                motor_direction_tx.send(next_direction).unwrap();
                                direction = next_direction;
                                behavior = Behavior::Moving
                            },
                            _ => ()
                            
                        }
                        
                    },
                    _ => (),
                }
            },
        }

        // This part is run to get the latest update of states, no matter which recv() was used.
        let behavior_str = match behavior {
            Behavior::Idle => "idle",
            Behavior::Moving => "moving",
            Behavior::DoorOpen => "doorOpen",
        }; // State is converted to string such that we don't need the state enum dependency at "network"
        // And also, the string is needed for the hall request assigner. 

        elevator_status_tx.send(ElevatorStatus {
            orders: orders.clone(),
            behavior: String::from(behavior_str),
            floor, // This one is presumed to be the same as the one above
            direction,
        }).unwrap();
    }
}