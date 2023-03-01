use std::net;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct HallOrder {
    address: String,
    floor: u8,
    call: u8
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct ElevatorState {
    address: String,
    behavior: String,
    floor: u8,
    direction: String,
    cab_requests: [bool; 4]
}

pub fn my_addr()-> String {
    let local_ip = net::TcpStream::connect("8.8.8.8:53")
        .unwrap()
        .local_addr()
        .unwrap()
        .ip();
    format!("{}",local_addr)
}