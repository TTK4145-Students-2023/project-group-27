use std::io::{stdout, Stdout, Write};

use crossbeam_channel::{Receiver, select};
use crossterm::{cursor, terminal, Result, ExecutableCommand};
use driver_rust::elevio::elev;

use crate::config;

const STATUS_SIZE: u16 = 24;

pub fn main(
    orders_rx: Receiver<[[bool; config::ELEV_NUM_BUTTONS as usize]; config::ELEV_NUM_FLOORS as usize]>,
    elevator_state_rx: Receiver<(String, u8, u8)>,
) -> Result<()> {
    let mut stdout = stdout();

    let mut state = (String::from("idle"), 0, 0);
    let mut orders = [[false; config::ELEV_NUM_BUTTONS as usize]; config::ELEV_NUM_FLOORS as usize];

    for _ in 0..STATUS_SIZE { writeln!(stdout, "")?; }

    loop {
        select! {
            recv(orders_rx) -> msg => {
                orders = msg.unwrap();
                printstatus(&mut stdout, orders, state.clone())?;
            },
            recv(elevator_state_rx) -> msg => {
                state = msg.unwrap();
                printstatus(&mut stdout, orders, state.clone())?;
            },
        }
    }
}

fn printstatus(
    stdout: &mut Stdout,
    orders: [[bool; config::ELEV_NUM_BUTTONS as usize]; config::ELEV_NUM_FLOORS as usize],
    state: (String, u8, u8),
) -> Result<()> {
    stdout.execute(cursor::MoveUp(STATUS_SIZE))?;
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    writeln!(stdout, "+---------------------------------------------------+")?;
    writeln!(stdout, "| ORDERS FOR THIS ELEVATOR                          |")?;
    writeln!(stdout, "+------------+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} |", "FLOOR", "HALL UP", "HALL DOWN", "CAB")?;
    for i in (0..config::ELEV_NUM_FLOORS).rev() {
        writeln!(stdout, "+------------+------------+------------+------------+")?;
        writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} |", i, orders[i as usize][0],  orders[i as usize][1],  orders[i as usize][2])?;
    }
    writeln!(stdout, "+------------+------------+------------+------------+\n\n")?;

    writeln!(stdout, "+-------------------------+")?;
    writeln!(stdout, "| STATE MACHINE           |")?;
    writeln!(stdout, "+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} |", "STATE", state.0)?;
    writeln!(stdout, "+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} |", "FLOOR", state.1)?;
    writeln!(stdout, "+------------+------------+")?;
    let dirn = match state.2 {
        elev::DIRN_UP   => "up",
        elev::DIRN_DOWN => "down",
        _ => "none",
    };
    writeln!(stdout, "| {0:<10} | {1:<10} |", "DIRECTION", dirn)?;
    writeln!(stdout, "+------------+------------+")?;

    Ok(())
}
