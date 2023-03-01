use crossbeam_channel::{select};
use std::time::Duration;
use std::thread;

pub mod config;
pub mod network;
pub mod orders;
pub mod test_hall_assigner;

fn main() {
    println!("Master started");
    
    let (receive_hall_order_rx, _receive_elevator_state_rx) = network::init();

    println!("Hello");
    thread::spawn(move || orders::main());

    test_hall_assigner::test_hall_assigner();

    loop {
        select! {
            recv(receive_hall_order_rx) -> msg => {
                let address = msg.clone().unwrap().address;
                let destination = msg.clone().unwrap().floor;
                let button = msg.unwrap().call;

                println!("received order | address {}, destination {}, button {}",address,destination,button);
            }
        }
    }
}
