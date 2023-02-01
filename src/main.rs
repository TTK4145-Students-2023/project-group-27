use driver_rust::elevio::elev as e;

fn main() -> std::io::Result<()> {
    let num_floors = 4;
    let elevator = e::Elevator::init("localhost:15657", num_floors)?;
    
    elevator.motor_direction(e::DIRN_UP);
    loop {
        
    }
}