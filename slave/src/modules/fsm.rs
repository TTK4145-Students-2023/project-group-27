/// ----- FSM MODULE -----
/// This module is the finite state machine controlling the elevator.
/// It receives events from other modules and switches states based
/// on these.

use std::time::Duration;

use crossbeam_channel::{select, Receiver, Sender, tick};

use shared_resources::config::ElevatorConfig;
use shared_resources::call::Call;
use shared_resources::request::Request;

use crate::utilities::direction::Direction;
use crate::utilities::elevator_status::{ElevatorStatus, Behaviour};
use crate::utilities::master_message::MasterMessage;

pub fn main(
    backup_data: ElevatorStatus,
    backup_send_tx: Sender<ElevatorStatus>,
    elevator_settings: ElevatorConfig,
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
    let timer = tick(Duration::from_secs_f64(0.25));

    let num_floors = elevator_settings.num_floors;
    let mut elevator = backup_data;

    loop {
        select! {
            recv(cab_button_rx) -> msg => {
                let destination = msg.unwrap();
                elevator.behaviour = match elevator.behaviour {
                    Behaviour::Moving => {
                        elevator.requests.add_order(destination, Call::Cab);
                        button_light_tx.send((Request{ floor: destination, call: Call::Cab }, true)).unwrap();
                        elevator.behaviour
                    },
                    Behaviour::Idle | Behaviour::DoorOpen => {
                        if elevator.floor == destination {
                            doors_activate_tx.send(true).unwrap();
                            Behaviour::DoorOpen
                        }
                        else {
                            elevator.requests.add_order(destination, Call::Cab);
                            button_light_tx.send((Request{ floor: destination, call: Call::Cab }, true)).unwrap();
                            elevator.behaviour
                        }
                    },
                }
            },
            recv(master_hall_requests_rx) -> msg => {
                let message = msg.unwrap();
                elevator.requests.update_hall_requests(message.our_hall_requests);
                for floor in 0..num_floors {
                    for call in Call::iter_hall() {
                        button_light_tx.send((
                            Request{ floor: floor, call: call }, 
                            message.all_hall_requests[floor as usize][call as usize],
                        )).unwrap();
                    }
                }
            },
            recv(floor_sensor_rx) -> msg => {
                elevator.floor = msg.unwrap();
                floor_indicator_tx.send(elevator.floor).unwrap();
                if elevator.should_stop() {
                    elevator.behaviour = match elevator.behaviour {
                        Behaviour::Idle | Behaviour::Moving => {
                            motor_direction_tx.send(Direction::Stop).unwrap();
                            doors_activate_tx.send(true).unwrap();
                            elevator.serve_requests_here();
                            button_light_tx.send((Request { floor: elevator.floor, call: Call::Cab }, false)).unwrap();
                            Behaviour::DoorOpen
                        },
                        Behaviour::DoorOpen => elevator.behaviour,
                    }
                }
            },
            recv(doors_closing_rx) -> _ => {
                elevator.behaviour = match elevator.behaviour {
                    Behaviour::DoorOpen => Behaviour::Idle,
                    Behaviour::Idle | Behaviour::Moving => elevator.behaviour,
                }
            },
            recv(timer) -> _ => {
                elevator.behaviour = match elevator.behaviour {
                    Behaviour::Idle => {
                        let next_direction = elevator.next_direction();
                        if next_direction.is_some() {
                            motor_direction_tx.send(next_direction.unwrap()).unwrap();
                            elevator.direction = next_direction.unwrap();
                            Behaviour::Moving
                        } else if elevator.current_floor_has_requests()
                            && elevator.should_stop() {
                            doors_activate_tx.send(true).unwrap();
                            elevator.serve_requests_here();
                            button_light_tx.send((Request { floor: elevator.floor, call: Call::Cab }, false)).unwrap();
                            Behaviour::DoorOpen
                        } else {
                            elevator.behaviour
                        }
                    },
                    Behaviour::DoorOpen => {
                        //if elevator.current_floor_has_requests() {
                        //    doors_activate_tx.send(true).unwrap();
                        //    elevator.serve_requests_here();
                        //    button_light_tx.send((Request { floor: elevator.floor, call: Call::Cab }, false)).unwrap();
                        //}
                        Behaviour::DoorOpen
                    },
                    Behaviour::Moving => elevator.behaviour,
                }
            }
        }
        elevator_status_tx.send(elevator.clone()).unwrap();

        backup_send_tx.send(elevator.clone()).unwrap();
    }
}
