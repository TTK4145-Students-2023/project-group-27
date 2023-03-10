use std::time::Duration;

use crossbeam_channel::{select, Sender, Receiver};

pub fn main(
    doors_closing_tx: Sender<bool>, 
    doors_activate_rx: Receiver<bool>, 
    obstruction_rx: Receiver<bool>,
    door_light_tx: Sender<bool>
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
            recv(doors_activate_rx) -> _ => {
                active = true;
                door_light_tx.send(true).unwrap();
            },
            default(Duration::from_secs_f64(TIMER_DURATION)) => {
                if active {
                    doors_closing_tx.send(true).unwrap();
                    active = false;
                    door_light_tx.send(false).unwrap();
                }
            },
        }
    }
}
