use std::io::Result;
use std::thread::spawn;
use std::process;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossbeam_channel as cbc;
use network_rust::udpnet;

use crate::config;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct OrderService {
    floor: u8,
    btn: u8,
    elevator_addr: String
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct DistributedOrders {
    elevators: Vec<Elevator>
}

impl DistributedOrders {
    pub fn init() -> Result<DistributedOrders> {
        Ok(Self {
            elevators: Vec::new()
        })
    }

    pub fn distribute_order(order_service: OrderService) {

    }

    fn add_elevator(addr: String) {

    }

    pub fn remove_elevator(addr: String) {

    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Elevator {
    addr: String,
    orders: [[bool; config::ELEV_NUM_BUTTONS as usize]; config::ELEV_NUM_FLOORS as usize]
}

pub fn main() {
    let distributed_orders: DistributedOrders = DistributedOrders::init().unwrap();
    
    let timeout_duration: u64 = 5;
    let mut last_seen: HashMap<String, Instant> = HashMap::new();
    spawn(move || {
        for (addr, instant) in last_seen {
            if Instant::now().duration_since(instant) > Duration::from_secs(timeout_duration) {

            }
        }
    });

    // RECEIVER FOR ORDER SERVICES
    let (order_service_recv_tx, order_service_recv_rx) = cbc::unbounded::<OrderService>();
    spawn(move || {
        if udpnet::bcast::rx(config::PORT, order_service_recv_tx).is_err() { process::exit(1) };
    });

    loop {
        cbc::select! {
            recv(order_service_recv_rx) -> order_service => {
                let addr = order_service.unwrap().elevator_addr;
                last_seen.insert(addr, Instant::now());
            }
        }
    }
}
