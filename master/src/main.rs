use crossbeam_channel::{select};
use std::time::Duration;

pub mod config;
// pub mod orders;
pub mod network;


fn main() {
    println!("Master started");
    
    let (receive_hall_order_rx, _receive_elevator_state_rx) = network::init();

    println!("Hello");

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
