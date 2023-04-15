/// ----- FSM MODULE -----
/// This module is the finite state machine controlling the elevator.
/// It receives events from other modules and switches states based
/// on these.

use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender, tick, unbounded};

use shared_resources::call::Call;
use shared_resources::request::Request;

use crate::utilities::direction::Direction;
use crate::utilities::elevator_status::{ElevatorStatus, Behaviour};
use crate::utilities::master_message::MasterMessage;

pub fn main(
    backup_data: ElevatorStatus,
    floor_sensor_rx: Receiver<u8>,
    floor_indicator_tx: Sender<u8>,
    button_light_tx: Sender<(Request,bool)>,
    doors_closing_rx: Receiver<bool>,
    doors_activate_tx: Sender<bool>,
    cab_button_rx: Receiver<u8>,
    motor_direction_tx: Sender<Direction>,
    master_hall_requests_rx: Receiver<MasterMessage>,
    elevator_status_tx: Sender<ElevatorStatus>,
) {
    let timer = tick(Duration::from_secs_f64(0.1));
    let (new_request_tx, new_request_rx) = unbounded();

    let mut elevator = backup_data;

    if elevator.behaviour == Behaviour::Moving {
        motor_direction_tx.send(elevator.direction).unwrap();
    } else if elevator.behaviour == Behaviour::DoorOpen {
        doors_activate_tx.send(true).unwrap();
    }

    loop {
        select! {
            // channels for receiving requests from other modules => generates the new_request event
            recv(cab_button_rx) -> msg => {
                let destination = msg.unwrap();
                elevator.requests.add_request(destination, Call::Cab);
                new_request_tx.send(true).unwrap();
            },
            recv(master_hall_requests_rx) -> msg => {
                let message = msg.unwrap();
                elevator.requests.update_hall_requests(message.our_hall_requests);
                if elevator.requests.has_unserved_requests() {
                    new_request_tx.send(true).unwrap();
                }
                for request_pair in elevator.requests.request_pair_iterator() {
                    button_light_tx.send(request_pair).unwrap();
                }
            },
            // channels for events in the state machine
            recv(new_request_rx) -> _ => {
                for request_pair in elevator.requests.request_pair_iterator() {
                    button_light_tx.send(request_pair).unwrap();
                }
                elevator.behaviour = match elevator.behaviour {
                    Behaviour::Idle => {
                        elevator.update_direction();
                        if elevator.requests_at_this_floor() {
                            doors_activate_tx.send(true).unwrap();
                            elevator.serve_requests_here();
                            button_light_tx.send((Request {
                                floor: elevator.floor,
                                call: Call::Cab
                            }, false)).unwrap();
                            button_light_tx.send((Request { 
                                floor: elevator.floor, 
                                call: elevator.direction.to_call().unwrap()
                            }, false)).unwrap();
                            Behaviour::DoorOpen
                        } else if elevator.requests.has_unserved_requests() {
                            motor_direction_tx.send(elevator.direction).unwrap();
                            Behaviour::Moving
                        } else { Behaviour::Idle }
                    },
                    _ => elevator.behaviour,
                };
            },
            recv(floor_sensor_rx) -> msg => {
                elevator.floor = msg.unwrap();
                floor_indicator_tx.send(elevator.floor).unwrap();
                if elevator.should_stop() {
                    elevator.behaviour = match elevator.behaviour {
                        Behaviour::Moving => {
                            motor_direction_tx.send(Direction::Stop).unwrap();
                            doors_activate_tx.send(true).unwrap();
                            elevator.serve_requests_here();
                            button_light_tx.send((Request { 
                                floor: elevator.floor, 
                                call: Call::Cab 
                            }, false)).unwrap();
                            button_light_tx.send((Request { 
                                floor: elevator.floor, 
                                call: elevator.direction.to_call().unwrap()
                            }, false)).unwrap();
                            Behaviour::DoorOpen
                        },
                        _ => elevator.behaviour,
                    }
                }
            },
            recv(doors_closing_rx) -> _ => {
                elevator.behaviour = match elevator.behaviour {
                    Behaviour::DoorOpen => {
                        elevator.update_direction();
                        if elevator.should_stop() && elevator.requests_at_this_floor() {
                            doors_activate_tx.send(true).unwrap();
                            elevator.serve_requests_here();
                            button_light_tx.send((Request {
                                floor: elevator.floor,
                                call: Call::Cab
                            }, false)).unwrap();
                            button_light_tx.send((Request { 
                                floor: elevator.floor, 
                                call: elevator.direction.to_call().unwrap()
                            }, false)).unwrap();
                            Behaviour::DoorOpen
                        } else if elevator.requests.has_unserved_requests() {
                            motor_direction_tx.send(elevator.direction).unwrap();
                            Behaviour::Moving
                        } else {
                            Behaviour::Idle
                        }
                    },
                    Behaviour::Idle | Behaviour::Moving => elevator.behaviour,
                }
            },
            recv(timer) -> _ => (),
        }
        elevator_status_tx.send(elevator.clone()).unwrap();
    }
}
