use crossbeam_channel::{unbounded, Receiver};
use std::time::Duration;
use std::thread::spawn;

use driver_rust::elevio::poll;
use driver_rust::elevio::elev;

pub fn init(elevator: elev::Elevator, poll_period: Duration) -> (
    Receiver<poll::CallButton>, 
    Receiver<u8>,
    Receiver<bool>,
    Receiver<bool>
) {
    let (call_button_tx, call_button_rx) = unbounded::<poll::CallButton>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::call_buttons(elevator, call_button_tx, poll_period));
    }

    let (floor_sensor_tx, floor_sensor_rx) = unbounded::<u8>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::floor_sensor(elevator, floor_sensor_tx, poll_period));
    }

    let (stop_button_tx, stop_button_rx) = unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::stop_button(elevator, stop_button_tx, poll_period));
    }

    let (obstruction_tx, obstruction_rx) = unbounded::<bool>();
    {
        let elevator = elevator.clone();
        spawn(move || poll::obstruction(elevator, obstruction_tx, poll_period));
    }
    (call_button_rx, floor_sensor_rx, stop_button_rx, obstruction_rx)
}

