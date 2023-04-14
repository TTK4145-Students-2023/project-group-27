/// ----- DOORS MODULE -----
/// This module is responsible for managing the door open light
/// and keeping track of how long the door has been open for, to
/// generate the doorClosing event for the state machine.

use std::time::Duration;

use crossbeam_channel::{select, Sender, Receiver};

pub fn main(
    obstruction_rx: Receiver<bool>,
    doors_activate_rx: Receiver<bool>,
    doors_closing_tx: Sender<bool>, 
    door_light_tx: Sender<bool>
) {
    const TIMER_DURATION: f64 = 3.0;
    let mut active: bool = false;

    loop {
        select! {
            recv(obstruction_rx) -> msg => {
                // received obstruction -> block this thread if doors are open
                active = msg.unwrap();
            },
            recv(doors_activate_rx) -> _ => {
                door_light_tx.send(true).unwrap();
            },
            default(Duration::from_secs_f64(TIMER_DURATION)) => {
                if !active {
                    doors_closing_tx.send(true).unwrap();
                    door_light_tx.send(false).unwrap();
                }
            },
        }
    }
}
