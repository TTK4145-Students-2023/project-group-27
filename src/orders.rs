use driver_rust::elevio::elev;
use driver_rust::elevio::poll;
use std::sync::{Arc, Mutex};

const orders: Arc<Mutex<Box<[u8]>>> = Arc::new(Mutex::new(Box::new()));