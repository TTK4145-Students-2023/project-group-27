/// ----- I/O MODULE -----
/// This module is responsible for polling the sensors on the elevator,
/// returning channels for other modules to listen to, as well as channels
/// for sending commands to the elevator driver. 

use std::time::Duration;
use std::thread::spawn;

use crossbeam_channel::{unbounded, Sender, Receiver};
use driver_rust::elevio::{poll, elev};

use shared_resources::config;
use shared_resources::call::Call;
use shared_resources::request::Request;

use crate::utilities::direction::Direction;

pub fn init(
    server_config: config::ServerConfig,
    elevator_settings: config::ElevatorConfig,
) -> (
    Receiver<u8>, 
    Receiver<Request>, 
    Receiver<u8>,
    Receiver<bool>,
    Receiver<bool>,
    Sender<(Request,bool)>,
    Sender<Direction>,
    Sender<bool>,
    Sender<u8>,
) {
    let serveraddr = "localhost:".to_owned() + &server_config.port.to_string();
    let elevator = elev::Elevator::init(serveraddr.as_str(), elevator_settings.num_floors).unwrap();

    let poll_period = Duration::from_millis(25);
    let (cab_button_tx, cab_button_rx) = unbounded();
    let (hall_button_tx, hall_button_rx) = unbounded();
    {
        let (call_button_tx, call_button_rx) = unbounded();
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
        spawn(move || { loop {
            let button_call = Request::from_elev(call_button_rx.recv().unwrap());
            match button_call.call {
                Call::Cab => cab_button_tx.send(button_call.floor).unwrap(),
                _ => hall_button_tx.send(button_call).unwrap(),
            }
        }});
    }

    let (floor_sensor_tx, floor_sensor_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
    }

    let (button_light_tx, button_light_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || { loop {
            let (request, on): (Request, bool) = button_light_rx.recv().unwrap();
            elevator.call_button_light(request.floor, request.call.as_elev_constant(), on);
        }});
    }

    let (motor_direction_tx, motor_direction_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || { loop {
            let dirn: Direction = motor_direction_rx.recv().unwrap();
            elevator.motor_direction(dirn.as_elev_constant());
        }});
    }

    let (door_light_tx, door_light_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || { loop {
            let on = door_light_rx.recv().unwrap();
            elevator.door_light(on);
        }});
    }
    door_light_tx.send(false).unwrap();

    let (floor_indicator_tx, floor_indicator_rx) = unbounded();
    {
        let elevator = elevator.clone();
        spawn(move || { loop {
            let on = floor_indicator_rx.recv().unwrap();
            elevator.floor_indicator(on);
        }});
    }

    // DRIVE ELEVATOR TO FLOOR
    if elevator.floor_sensor().is_none() {
        motor_direction_tx.send(Direction::Down).unwrap();
    }
    
    (cab_button_rx, 
     hall_button_rx, 
     floor_sensor_rx, 
     stop_button_rx, 
     obstruction_rx,
     button_light_tx,
     motor_direction_tx,
     door_light_tx,
     floor_indicator_tx)
}