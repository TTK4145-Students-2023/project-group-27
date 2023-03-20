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

#[derive(Clone, Debug)]
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

fn update_hall_orders(
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

fn update_cab_requests(
    orders: &Vec<Vec<bool>>,
    n_floors: u8
) -> Vec<bool> {
    let mut cab_requests = vec![false; n_floors as usize];
    for floor in 0..n_floors {
        cab_requests[floor as usize] = orders[floor as usize][elev::CAB as usize];
    }
    cab_requests
}

pub fn main (
    elevator_settings: ElevatorSettings,
    cab_button_rx: Receiver<poll::CallButton>, // From io
    hall_requests_rx: Receiver<HallRequests>, //From network
    doors_closing_rx: Receiver<bool>, // From doors
    floor_sensor_rx: Receiver<u8>, // From io
    cab_requests_tx: Sender<Vec<bool>>, // To network
    button_light_tx: Sender<(u8,u8,bool)>, // To io
    doors_activate_tx: Sender<bool>, // To doors
    motor_direction_tx: Sender<u8>, // To io
    floor_indicator_tx: Sender<u8>, // To io
    cleared_request_tx: Sender<poll::CallButton>, // To network
    elevator_status_tx: Sender<ElevatorStatus> // To network & debug
) -> std::io::Result<()> {

    let timer = tick(Duration::from_secs_f64(0.1));
    let n_buttons = elevator_settings.num_buttons;
    let n_floors = elevator_settings.num_floors;

    let mut floor: u8 = 0;
    let mut direction: u8 = elev::DIRN_DOWN;
    let mut behavior: Behavior = Behavior::Moving;
    let mut orders = vec![vec![false; n_buttons as usize]; n_floors as usize];
    let mut cab_requests = vec![false; n_floors as usize];
    let mut destination: u8;

    // CLEAR ALL LIGHTS
    for floor in 0..n_floors {
        for btn in elev::HALL_UP..=elev::CAB {
            button_light_tx.send((floor, btn, false)).unwrap(); //Would it be nice to rather send a matrix to IO and then do the for looping at IO?
        }
    }

    loop {
        select! {
            recv(cab_button_rx) -> cb_msg => {
                destination = cb_msg.as_ref().unwrap().floor;
                match behavior {
                    Behavior::Idle => {
                        if floor == destination {
                            behavior = Behavior::DoorOpen;
                            doors_activate_tx.send(true).unwrap();
                        }
                        else {
                            orders[destination as usize][elev::CAB as usize] = true;
                            button_light_tx.send((destination, elev::CAB, true)).unwrap();
                            cab_requests = update_cab_requests(&orders,n_floors);
                        }
                    },
                    Behavior::Moving => {
                        orders[destination as usize][elev::CAB as usize] = true;
                        button_light_tx.send((destination, elev::CAB, true)).unwrap();
                        cab_requests = update_cab_requests(&orders,n_floors);
                    },
                    Behavior::DoorOpen => {
                        if floor == destination {
                            doors_activate_tx.send(true).unwrap();
                        }
                        else {
                            orders[destination as usize][elev::CAB as usize] = true;
                            button_light_tx.send((destination, elev::CAB, true)).unwrap();
                            cab_requests = update_cab_requests(&orders,n_floors);
                        }
                    },
                }
            },
            recv(hall_requests_rx) -> hr_msg => {
                let all_requests = hr_msg.clone().unwrap().all_requests;
                let our_requests = hr_msg.clone().unwrap().our_requests;
                match behavior {
                    Behavior::Moving => {
                        orders = update_hall_orders(n_floors, n_buttons, hr_msg.unwrap(), button_light_tx.clone());
                    },
                    _ => {
                        for f in 0..n_floors {
                            for b in elev::HALL_UP..=elev::HALL_DOWN {
                                if (our_requests[f as usize][b as usize] && f == floor) 
                                && (!further_requests_in_direction(n_floors, &orders, floor, direction) 
                                || cab_requests[f as usize] == orders[f as usize][b as usize]) {
                                    behavior = Behavior::DoorOpen;
                                    doors_activate_tx.send(true).unwrap();
                                    button_light_tx.send((floor, elev::CAB, false)).unwrap();
                                    orders[f as usize][b as usize] = false;
                                    cab_requests[f as usize] = false;
                                    cleared_request_tx.send(poll::CallButton {
                                    floor: f,
                                    call: if direction == elev::DIRN_UP { elev::HALL_UP } else { elev::HALL_DOWN },
                                    }).unwrap();
                                    cleared_request_tx.send(poll::CallButton {
                                        floor: floor,
                                        call: if direction == elev::DIRN_UP { elev::HALL_DOWN } else { elev::HALL_UP },
                                    }).unwrap();
                                }
                                else {
                                    button_light_tx.send((f, b, all_requests[f as usize][b as usize])).unwrap();
                                    orders[f as usize][b as usize] = our_requests[f as usize][b as usize];
                                }
                            }   
                        }
                    }
                }
            },
            recv(floor_sensor_rx) -> fs_msg => {
                floor = fs_msg.unwrap();
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
                    _ => panic!("new floor was reached even tough elevator was not moving"), // This case should not be possible as a still elevator does not reach a new floor
                }
            },
            recv(doors_closing_rx) -> _ => {
                match behavior {
                    Behavior::DoorOpen => {
                        behavior = Behavior::Idle;
                    },
                    _ => panic!("doors were closing even though it was not open"), // Should not be possible to recieve this while open
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

        let behavior_str = match behavior {
            Behavior::Idle => "idle",
            Behavior::Moving => "moving",
            Behavior::DoorOpen => "doorOpen",
        };
        elevator_status_tx.send(ElevatorStatus {
            orders: orders.clone(),
            behavior: String::from(behavior_str),
            floor,
            direction,
        }).unwrap();

        cab_requests_tx.send(cab_requests.clone()).unwrap();
    }
}