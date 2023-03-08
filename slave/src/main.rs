use crossbeam_channel::select;

pub mod doors;
pub mod io;
pub mod fsm;
pub mod requests;
pub mod config;
pub mod network;

fn main() -> std::io::Result<()> {
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
    ) = io::init();
    println!("module initialized: inputs");

    // INITIALIZE DOORS MODULE
    let (
        doors_activate_tx, 
        doors_closing_rx
    ) = doors::init(
        obstruction_rx,
        door_light_tx
    );
    println!("module initialized: doors");

    // INITIALIZE REQUESTS MODULE
    let (
        should_stop_rx, 
        next_direction_rx, 
        hall_request_tx,
        cab_requests_rx,
        elevator_data_tx
    ) = requests::init(
        cab_button_rx,
        button_light_tx,
    );
    println!("module initialized: requests");

    // INITIALIZE FSM MODULE
    let elevator_state_rx = fsm::init(
        should_stop_rx, 
        doors_activate_tx,
        next_direction_rx,
        doors_closing_rx,
        floor_sensor_rx,
        floor_indicator_tx,
        motor_direction_tx,
        elevator_data_tx
    );
    println!("module initialized: fsm");

    // INITIALIZE NETWORK MODULE
    network::init(
        hall_button_rx,
        hall_request_tx,
        elevator_state_rx,
        cab_requests_rx,
    );
    println!("module initialized: network");

    loop {
        select! {
            recv(stop_button_rx) -> _ => {
                println!("STOPPING PROGRAM...");
                return Ok(())
            }
        }
    }
}
