use crossbeam_channel::{unbounded, select, Sender, Receiver};
use std::time::Duration;
use std::thread::spawn;

pub fn init() -> (Sender<bool>, Receiver<bool>) {
    let (doors_activate_tx, doors_activate_rx) = unbounded();
    let (doors_timed_out_tx, doors_timed_out_rx) = unbounded();
    spawn(move || main(doors_timed_out_tx, doors_activate_rx));
    (doors_activate_tx, doors_timed_out_rx)
}

fn main(s: crossbeam_channel::Sender<bool>, r: crossbeam_channel::Receiver<bool>) {
    const TIMER_DURATION: f64 = 3.0;
    let mut active: bool = false;

    loop {
        select! {
            recv(r) -> msg => {
                match msg.unwrap() {
                    true => active = true,
                    false => active = false
                }
            },
            default(Duration::from_secs_f64(TIMER_DURATION)) => {
                if active {
                    s.send(true).unwrap();
                    active = false;
                }
            },
        }
    }
}
