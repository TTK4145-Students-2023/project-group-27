use std::thread;

use crossbeam_channel::{select, unbounded};

pub mod doors;
pub mod io;
//pub mod fsm;
//pub mod requests;
pub mod config;
pub mod network;
pub mod debug;
pub mod prototype_fsm;

fn main() -> std::io::Result<()> {
    // READ CONFIGURATION
    let config = config::get_config();

    // INITIALIZE CHANNELS
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    //let (should_stop_tx, should_stop_rx) = unbounded();
    //let (next_direction_tx, next_direction_rx) = unbounded();
    //let (our_hall_requests_tx, our_hall_requests_rx) = unbounded();
    //let (all_hall_requests_tx, all_hall_requests_rx) = unbounded();
    let (hall_requests_tx, hall_requests_rx) = unbounded();
    let (cleared_request_tx, cleared_request_rx) = unbounded();
    let (cab_requests_tx, cab_requests_rx) = unbounded();
    //let (elevator_data_tx, elevator_data_rx) = unbounded();
    //let (elevator_state_tx, elevator_state_rx) = unbounded();
    //let (orders_tx, orders_rx) = unbounded();
    let (elevator_status_tx, elevator_status_rx) = unbounded();

    // INITIALIZE INPUTS MODULE
    let (
        cab_button_rx, 
        hall_button_rx, 
        floor_sensor_rx, 
        stop_button_rx, 
        obstruction_rx,
        button_light_tx,
        motor_direction_tx,
        door_light_tx,
        floor_indicator_tx,
    ) = io::init(
        config.server,
        config.settings.clone(),
    );

    // INITIALIZE THREAD FOR DOOR EVENTS
    thread::spawn(move || doors::main(
        obstruction_rx,
        doors_activate_rx,
        doors_closing_tx,
        door_light_tx
    ));

    // INITIALIZE THREAD FOR REQUEST EVENTS
    // {
    //     let elevator_settings = config.settings.clone();
    //     thread::spawn(move || requests::main(
    //         elevator_settings,
    //         cab_button_rx, 
    //         our_hall_requests_rx,
    //         all_hall_requests_rx,
    //         elevator_data_rx,
    //         cleared_request_tx,
    //         button_light_tx,
    //         should_stop_tx,
    //         next_direction_tx,
    //         cab_requests_tx,
    //         orders_tx,
    //     ));
    // }

    // INITIALIZE THREAD FOR STATE MACHINE
    // thread::spawn(move || fsm::main(
    //     should_stop_rx,
    //     next_direction_rx,
    //     doors_closing_rx,
    //     floor_sensor_rx,
    //     doors_activate_tx,
    //     motor_direction_tx,
    //     floor_indicator_tx,
    //     elevator_state_tx,
    //     elevator_data_tx,
    // ));

    // INITIALIZE THREAD FOR PROTOTYPE FSM
    let elevator_settings = config.settings.clone();
    thread::spawn(move || prototype_fsm::main(
        elevator_settings,
        cab_button_rx,
        hall_requests_rx,
        doors_closing_rx,
        floor_sensor_rx,
        cab_requests_tx,
        button_light_tx,
        doors_activate_tx,
        motor_direction_tx,
        floor_indicator_tx,
        cleared_request_tx,
        elevator_status_tx,
    ));

    // INITIALIZE NETWORK MODULE
    {
        let elevator_status_rx = elevator_status_rx.clone();
        let elevator_settings = config.settings.clone();
        thread::spawn(move || network::main(
            elevator_settings,
            config.network,
            hall_button_rx,
            cleared_request_rx,
            elevator_status_rx,
            cab_requests_rx,
            hall_requests_tx,
        ));
    }

    // INITIALIZE DEBUG MODULE
    {
        let elevator_settings = config.settings.clone();
        thread::spawn(move || debug::main(
            elevator_settings,
            elevator_status_rx,
        ));
    }

    loop {
        select! {
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
