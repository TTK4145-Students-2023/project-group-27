use std::collections::HashMap;
use std::process::Command;
use std::thread::spawn;

use crossbeam_channel::{Receiver, Sender, unbounded, select};

use crate::config;
use crate::network::{HallOrder, ElevatorState};

const EXEC_NAME: &str = "hall_request_assigner";

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HRAElevState {
    behaviour: String, 
    floor: u8,    
    direction: String,  
    cab_requests: [bool; config::ELEV_NUM_FLOORS as usize]
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct HRAInput {
    hall_requests: [[bool; 2]; config::ELEV_NUM_FLOORS as usize],
    states: HashMap<String, HRAElevState>
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HRAOutput {
    elevator_orders: HashMap<String, [[bool; 2]; config::ELEV_NUM_FLOORS as usize]>
}

pub fn init(
    hall_order_rx: Receiver<HallOrder>, 
    elevator_state_rx: Receiver<ElevatorState>
) -> Receiver<HRAOutput> {
    let (assigned_requests_tx, assigned_requests_rx) = unbounded();
    spawn(move || main(
        hall_order_rx,
        elevator_state_rx,
        assigned_requests_tx
    ));
    assigned_requests_rx
}

fn main(
    hall_order_rx: Receiver<HallOrder>, 
    elevator_state_rx: Receiver<ElevatorState>,
    assigned_requests_tx: Sender<HRAOutput>
) {
    let mut hall_requests = [[false; 2]; config::ELEV_NUM_FLOORS as usize];
    let mut states = HashMap::new();

    loop {
        select!{
            recv(hall_order_rx) -> msg => {
                let floor = msg.clone().unwrap().floor;
                let call = msg.unwrap().call;
                hall_requests[floor as usize][call as usize] = true;
            },
            recv(elevator_state_rx) -> msg => {
                let addr = msg.clone().unwrap().id;
                let state = HRAElevState {
                    behaviour: msg.clone().unwrap().behaviour, 
                    floor: msg.clone().unwrap().floor,    
                    direction: msg.clone().unwrap().direction,  
                    cab_requests: msg.unwrap().cab_requests
                };
                states.insert(addr, state);
            },
        }
        let assigned_orders = assign_orders(hall_requests, states.clone());
        assigned_requests_tx.send(assigned_orders).unwrap();
    }
}

fn assign_orders(
    hall_requests: [[bool; 2]; config::ELEV_NUM_FLOORS as usize],
    states: HashMap<String, HRAElevState>
) -> HRAOutput {
    let input = HRAInput {
        hall_requests: hall_requests,
        states: states
    };
    let json_arg = serde_json::to_string(&input).unwrap();
    let command = "./".to_owned() + EXEC_NAME + " -i '" + json_arg.as_str() + "'";
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to call hall request assigner");
    let str_result = String::from_utf8(result.stdout).unwrap();
    let output: HRAOutput = serde_json::from_str(&str_result).unwrap();
    output
}
