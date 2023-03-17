/// ----- DEBUG MODULE -----
/// This module receives information about the state and orders
/// for this elevator and does a formatted print to the console.

use std::io::{stdout, Stdout, Write};

use crossbeam_channel::{Receiver, select};
use crossterm::{cursor, terminal, Result, ExecutableCommand};
use driver_rust::elevio::elev;

use crate::{config::ElevatorSettings, prototype_fsm::ElevatorStatus};

const STATUS_SIZE: u16 = 24;

pub fn main(
    elevator_settings: ElevatorSettings,
    //orders_rx: Receiver<Vec<Vec<bool>>>,
    //elevator_state_rx: Receiver<(String, u8, u8)>,
    elevator_status_rx: Receiver<ElevatorStatus>
) -> Result<()> {
    let mut stdout = stdout();

    let mut status = (String::from("idle"), 0, 0);
    let mut orders = vec![vec![false; elevator_settings.num_buttons as usize]; elevator_settings.num_floors as usize];

    for _ in 0..STATUS_SIZE { writeln!(stdout, "")?; }

    loop {
        select! {
            recv(elevator_status_rx) -> msg => {
                orders = msg.clone().unwrap().orders;
                status = (msg.clone().unwrap().state, msg.clone().unwrap().floor, msg.clone().unwrap().direction);
                printstatus(elevator_settings.clone(), &mut stdout, orders.clone(), status.clone())?;
                printstatus(elevator_settings.clone(), &mut stdout, orders, status)?;
            },
        }
    }
}

fn printstatus(
    elevator_settings: ElevatorSettings,
    stdout: &mut Stdout,
    orders: Vec<Vec<bool>>,
    state: (String, u8, u8),
) -> Result<()> {
    stdout.execute(cursor::MoveUp(STATUS_SIZE))?;
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown))?;

    writeln!(stdout, "+---------------------------------------------------+")?;
    writeln!(stdout, "| ORDERS FOR THIS ELEVATOR                          |")?;
    writeln!(stdout, "+------------+------------+------------+------------+")?;
    writeln!(stdout, "| {0:<10} | {1:<10} | {2:<10} | {3:<10} |", "FLOOR", "HALL UP", "HALL DOWN", "CAB")?;
    for i in (0..elevator_settings.num_floors).rev() {
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
