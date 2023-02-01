use driver_rust::elevio::elev as e;

fn main() -> std::io::Result<()> {
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