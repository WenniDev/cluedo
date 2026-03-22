use crate::{Message, encode};

use std::net::UdpSocket;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Sender};

pub const SOCKET_ADDR: &str = "127.0.0.1:7877";
static SOCKET: OnceLock<UdpSocket> = OnceLock::new();
static TX: OnceLock<Sender<Message>> = OnceLock::new();

fn send(msg: &Message) {
    if let Some(sock) = SOCKET.get() {
        if let Some(bytes) = encode(msg) {
            let _ = sock.send(&bytes);
        }
    }
}

pub fn enqueue(msg: Message) {
    if let Some(tx) = TX.get() {
        let _ = tx.send(msg);
    }
}

pub fn connect() -> Result<(), Box<dyn std::error::Error>> {
    let sock = UdpSocket::bind("127.0.0.1:0")?;
    sock.connect(SOCKET_ADDR)?;
    SOCKET.set(sock).ok();

    let (tx, rx) = mpsc::channel::<Message>();
    TX.set(tx).ok();

    std::thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            send(&msg);
        }
    });

    enqueue(Message::Connected);
    log::info!("connected to {SOCKET_ADDR}");
    Ok(())
}
