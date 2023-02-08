use std::thread::sleep;
use std::time::Duration;

use driver_rust::elevio::elev as e;

mod requests;
mod elevator_io_types;
mod elevator;
mod fsm;
pub mod timer;

fn main() -> std::io::Result<()> {
    
    let elevator = e::Elevator::init(elevator_io_types::ADDR, elevator_io_types::N_FLOORS)?;

    // TEST AV TIMER MODULEN
    let duration: f64 = 1.5;
    timer::timer_start(duration);
    print!("timer_timed_out(): {}\n", timer::timer_timed_out());
    sleep(Duration::from_secs(2));
    print!("timer_timed_out(): {}\n", timer::timer_timed_out());

    elevator.motor_direction(e::DIRN_UP);

    loop {
        match elevator.floor_sensor() {
            Some(0) => elevator.motor_direction(e::DIRN_UP),
            Some(3) => elevator.motor_direction(e::DIRN_DOWN),
            _ => if elevator.floor_sensor().unwrap_or(0) != 0 {
                println!("Current floor: {}",elevator.floor_sensor().unwrap());
            },
        }
    }
}