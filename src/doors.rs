use std::time::Duration;
use std::thread::spawn;

use crossbeam_channel::{unbounded, select, Sender, Receiver};

pub fn init(obstruction_rx: Receiver<bool>) -> (Sender<bool>, Receiver<bool>) {
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_closing_tx, doors_closing_rx) = unbounded();
    spawn(move || main(doors_closing_tx, doors_activate_rx, obstruction_rx));
    (doors_activate_tx, doors_closing_rx)
}

fn main(
    doors_closing_tx: Sender<bool>, 
    doors_activate_rx: Receiver<bool>, 
    obstruction_rx: Receiver<bool>
) {
    const TIMER_DURATION: f64 = 3.0;
    let mut active: bool = false;

    loop {
        select! {
            recv(obstruction_rx) -> msg => {
                if active && msg.unwrap() {
                    obstruction_rx.recv().unwrap(); // block until next message from obstruction
                }
            },
            recv(doors_activate_rx) -> msg => {
                match msg.unwrap() {
                    true => active = true,
                    false => active = false
                }
            },
            default(Duration::from_secs_f64(TIMER_DURATION)) => {
                if active {
                    doors_closing_tx.send(true).unwrap();
                    active = false;
                }
            },
        }
    }
}
