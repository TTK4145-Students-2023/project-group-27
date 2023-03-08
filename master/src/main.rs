pub mod config;
pub mod network;
pub mod test_hall_assigner;
pub mod hall_request_assigner;

fn main() {
    println!("Master started");
    
    network::main();
}
