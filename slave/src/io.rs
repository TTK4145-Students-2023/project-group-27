/// ----- I/O MODULE -----
/// This module is responsible for polling the sensors on the elevator,
/// returning channels for other modules to listen to, as well as channels
/// for sending commands to the elevator driver. 

use std::time::Duration;
use std::thread;

use crossbeam_channel::{unbounded, Sender, Receiver};
use driver_rust::elevio::{poll, elev};

use crate::config;

pub fn init(
    server_config: config::ServerConfig,
    elevator_settings: config::ElevatorSettings,
) -> (
    Receiver<poll::CallButton>, 
    Receiver<poll::CallButton>, 
    Receiver<u8>,
    Receiver<bool>,
    Receiver<bool>,
    Sender<(u8,u8,bool)>,
    Sender<u8>,
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
        thread::Builder::new().name("call_buttons".to_string()).spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period)).ok();
        thread::Builder::new().name("cab_button".to_string()).spawn(move || { loop {
            let button_call = call_button_rx.recv().unwrap();
            match button_call.call {
                elev::CAB => cab_button_tx.send(button_call).unwrap(),
                _ => hall_button_tx.send(button_call).unwrap(),
            }
        }}).ok();
    }

    let (floor_sensor_tx, floor_sensor_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("floor_sensor".to_string()).spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period)).ok();
    }

    let (stop_button_tx, stop_button_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("stop_button".to_string()).spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period)).ok();
    }

    let (obstruction_tx, obstruction_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("obstruction".to_string()).spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period)).ok();
    }

    let (button_light_tx, button_light_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("call_button_light".to_string()).spawn(move || { loop {
            let (floor, call, on) = button_light_rx.recv().unwrap();
            elevator.call_button_light(floor, call, on);
        }}).ok();
    }

    let (motor_direction_tx, motor_direction_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("motor_direction".to_string()).spawn(move || { loop {
            let dirn = motor_direction_rx.recv().unwrap();
            elevator.motor_direction(dirn);
        }}).ok();
    }

    let (door_light_tx, door_light_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("door_light".to_string()).spawn(move || { loop {
            let on = door_light_rx.recv().unwrap();
            elevator.door_light(on);
        }}).ok();
    }
    door_light_tx.send(false).unwrap();

    let (floor_indicator_tx, floor_indicator_rx) = unbounded();
    {
        let elevator = elevator.clone();
        thread::Builder::new().name("floor_indicator".to_string()).spawn(move || { loop {
            let on = floor_indicator_rx.recv().unwrap();
            elevator.floor_indicator(on);
        }}).ok();
    }

    // DRIVE ELEVATOR TO FLOOR
    if elevator.floor_sensor().is_none() {
        motor_direction_tx.send(elev::DIRN_DOWN).unwrap();
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