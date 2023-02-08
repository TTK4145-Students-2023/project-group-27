use std::thread::sleep;
use std::time::Duration;

use driver_rust::elevio::elev as e;

pub mod timer;

fn main() -> std::io::Result<()> {
    // TEST AV TIMER MODULEN
    let duration: f64 = 1.5;
    timer::timer_start(duration);
    print!("timer_timed_out(): {}\n", timer::timer_timed_out());
    sleep(Duration::from_secs(2));
    print!("timer_timed_out(): {}\n", timer::timer_timed_out());

    let num_floors = 4;
    let elevator = e::Elevator::init("localhost:15657", num_floors)?;

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