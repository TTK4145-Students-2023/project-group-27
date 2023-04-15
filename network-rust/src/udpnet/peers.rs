use std::collections::HashMap;
use std::str;
use std::time;

use crossbeam_channel as cbc;
use log::error;

#[path = "./sock.rs"]
mod sock;

#[derive(Debug)]
pub struct PeerUpdate {
    pub peers: Vec<String>,
    pub new: Option<String>,
    pub lost: Vec<String>,
}

/// Periodically broadcast `id` can be toggled with `tx_enable`
///
/// Returns `Err` when creating a socket fails. Ignores sending errors after
/// the socket has been created.
pub fn tx(port: u16, id: String, tx_enable: cbc::Receiver<bool>) -> std::io::Result<()> {
    let (s, addr) = sock::new_tx(port, false)?;

    let mut enabled = true;

    let ticker = cbc::tick(time::Duration::from_millis(15));

    loop {
        cbc::select! {
            recv(tx_enable) -> enable => {
                enabled = enable.unwrap();
            },
            recv(ticker) -> _ => {
                if enabled {
                    if let Err(e) = s.send_to(id.as_bytes(), &addr) {
                        error!("Sending failed: {}", e);
                    }
                }
            },
        }
    }
}

/// Forward `id`'s retrieved from the network on port `port`.
///
/// Returns `Err` when creating a socket fails. Ignores receiving errors after
/// creating a socket.
/// Panics if sending to the channel fails.
pub fn rx(port: u16, peer_update: cbc::Sender<PeerUpdate>) -> std::io::Result<()> {
    let timeout = time::Duration::from_millis(500);
    let s = sock::new_rx(port)?;
    s.set_read_timeout(Some(timeout))?;

    let mut last_seen: HashMap<String, time::Instant> = HashMap::new();
    let mut buf = [0; 1024];

    loop {
        let mut modified = false;
        let mut p = PeerUpdate {
            peers: Vec::new(),
            new: None,
            lost: Vec::new(),
        };

        let now = time::Instant::now();

        // Finding new peers
        if let Ok(n) = s.recv(&mut buf) {
            if let Ok(id) = str::from_utf8(&buf[..n]) {
                p.new = if !last_seen.contains_key(id) {
                    modified = true;
                    Some(id.to_string())
                } else {
                    None
                };
                last_seen.insert(id.to_string(), now);
            }
        }

        // Finding lost peers
        for (id, when) in &last_seen {
            if now - *when > timeout {
                p.lost.push(id.to_string());
                modified = true;
            }
        }
        //  .. and removing them
        for id in &p.lost {
            last_seen.remove(id);
        }

        // Sending peer update
        if modified {
            p.peers = last_seen.keys().cloned().collect();
            p.peers.sort();
            p.lost.sort();
            peer_update.send(p).unwrap();
        }
    }
}
