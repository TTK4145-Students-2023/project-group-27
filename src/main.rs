use driver_rust::elevio;

fn main() -> std::io::Result<()> {
    
    let num_floors = 4;
    let elevator = elevio::Elevator::init("localhost:15657", num_floors)?;
}