use driver_rust::elevio::elev as e;

mod requests;
mod elevator_io_types;
mod elevator;

fn main() -> std::io::Result<()> {
    
    let elevator = e::Elevator::init(elevator_io_types::ADDR, elevator_io_types::N_FLOORS)?;

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