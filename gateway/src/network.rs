use std::net::{TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::log::{LogEntry, format_message, log_tx, now_str};

pub type WsClients = Arc<Mutex<Vec<mpsc::Sender<Vec<u8>>>>>;

pub static STOP_UDP: AtomicBool = AtomicBool::new(false);
pub static STOP_WS: AtomicBool = AtomicBool::new(false);

pub static SONGS_PLAYED: AtomicU32 = AtomicU32::new(0);
pub static TOTAL_MISS: AtomicU32 = AtomicU32::new(0);
pub static TOTAL_FAST: AtomicU32 = AtomicU32::new(0);

pub static COUNT_MARVELOUS: AtomicU32 = AtomicU32::new(0);
pub static COUNT_PERFECT: AtomicU32 = AtomicU32::new(0);
pub static COUNT_GREAT: AtomicU32 = AtomicU32::new(0);
pub static COUNT_GOOD: AtomicU32 = AtomicU32::new(0);
pub static COUNT_OK: AtomicU32 = AtomicU32::new(0);
pub static COUNT_MISS: AtomicU32 = AtomicU32::new(0);
pub static COUNT_FAST: AtomicU32 = AtomicU32::new(0);
pub static COUNT_SLOW: AtomicU32 = AtomicU32::new(0);

pub fn start_udp_thread(addr: String, ws_clients: WsClients) {
    thread::spawn(move || {
        // Wait for the previous UDP thread to notice STOP_UDP and release the socket.
        thread::sleep(Duration::from_millis(150));
        STOP_UDP.store(false, Ordering::Relaxed);
        let sock = match UdpSocket::bind(&addr) {
            Ok(s) => {
                s.set_read_timeout(Some(Duration::from_millis(100))).ok();
                s
            }
            Err(e) => {
                log_tx()
                    .try_send(LogEntry {
                        time: now_str(),
                        msg: format!("DDR bind error {addr}: {e}"),
                    })
                    .ok();
                return;
            }
        };
        log_tx()
            .try_send(LogEntry {
                time: now_str(),
                msg: format!("DDR listening on {addr}"),
            })
            .ok();
        let mut buf = [0u8; 4096];
        while !STOP_UDP.load(Ordering::Relaxed) {
            match sock.recv(&mut buf) {
                Ok(len) => {
                    let data = buf[..len].to_vec();
                    if let Some(msg) = socket::decode(&data) {
                        match &msg {
                            socket::Message::Judgment(j) => {
                                use socket::JudgmentKind::*;
                                match j.kind {
                                    Marvelous => { COUNT_MARVELOUS.fetch_add(1, Ordering::Relaxed); }
                                    Perfect => { COUNT_PERFECT.fetch_add(1, Ordering::Relaxed); }
                                    Great => { COUNT_GREAT.fetch_add(1, Ordering::Relaxed); }
                                    Good => { COUNT_GOOD.fetch_add(1, Ordering::Relaxed); }
                                    Ok => { COUNT_OK.fetch_add(1, Ordering::Relaxed); }
                                    Miss | Ng => { COUNT_MISS.fetch_add(1, Ordering::Relaxed); }
                                }
                                match j.kind {
                                    Marvelous | Ok | Ng => {}
                                    _ => {
                                        if j.timing < 0 {
                                            COUNT_FAST.fetch_add(1, Ordering::Relaxed);
                                            TOTAL_FAST.fetch_add(1, Ordering::Relaxed);
                                        } else if j.timing > 0 {
                                            COUNT_SLOW.fetch_add(1, Ordering::Relaxed);
                                        }
                                    }
                                }
                                if matches!(j.kind, Miss | Ng) {
                                    TOTAL_MISS.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                            socket::Message::Next => {
                                SONGS_PLAYED.fetch_add(1, Ordering::Relaxed);
                                for c in [&COUNT_MARVELOUS, &COUNT_PERFECT, &COUNT_GREAT, &COUNT_GOOD, &COUNT_OK, &COUNT_MISS, &COUNT_FAST, &COUNT_SLOW] {
                                    c.store(0, Ordering::Relaxed);
                                }
                            }
                            _ => {}
                        }
                        log_tx().try_send(LogEntry { time: now_str(), msg: format_message(&msg) }).ok();
                    }
                    ws_clients
                        .lock()
                        .unwrap()
                        .retain(|tx| tx.send(data.clone()).is_ok());
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    log_tx()
                        .try_send(LogEntry {
                            time: now_str(),
                            msg: format!("DDR error: {e}"),
                        })
                        .ok();
                    break;
                }
            }
        }
    });
}

pub fn start_ws_thread(addr: String, ws_clients: WsClients) {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(150));
        STOP_WS.store(false, Ordering::Relaxed);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => {
                l.set_nonblocking(true).ok();
                l
            }
            Err(e) => {
                log_tx()
                    .try_send(LogEntry {
                        time: now_str(),
                        msg: format!("OBS bind error {addr}: {e}"),
                    })
                    .ok();
                return;
            }
        };
        log_tx()
            .try_send(LogEntry {
                time: now_str(),
                msg: format!("OBS listening on {addr}"),
            })
            .ok();
        while !STOP_WS.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nonblocking(false).ok();
                    let Ok(mut ws) = tungstenite::accept(stream) else {
                        continue;
                    };
                    let (tx, rx) = mpsc::channel::<Vec<u8>>();
                    ws_clients.lock().unwrap().push(tx);
                    thread::spawn(move || {
                        for data in rx {
                            if ws
                                .send(tungstenite::Message::Text(
                                    String::from_utf8_lossy(&data).into_owned().into(),
                                ))
                                .is_err()
                            {
                                break;
                            }
                        }
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    log_tx()
                        .try_send(LogEntry {
                            time: now_str(),
                            msg: format!("OBS error: {e}"),
                        })
                        .ok();
                    break;
                }
            }
        }
    });
}
