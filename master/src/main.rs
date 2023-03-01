use std::thread;

pub mod config;
pub mod orders;
pub mod test_hall_assigner;

fn main() {
    println!("Master started");
    
    thread::spawn(move || orders::main());

    test_hall_assigner::test_hall_assigner();

    loop {
        
    }
}
