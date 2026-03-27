use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use socket::Message;

pub struct LogEntry {
    pub time: String,
    pub msg: String,
}

static LOG_CHANNEL: OnceLock<(
    async_channel::Sender<LogEntry>,
    async_channel::Receiver<LogEntry>,
)> = OnceLock::new();

pub fn log_channel() -> &'static (
    async_channel::Sender<LogEntry>,
    async_channel::Receiver<LogEntry>,
) {
    LOG_CHANNEL.get_or_init(|| async_channel::unbounded())
}

pub fn log_tx() -> &'static async_channel::Sender<LogEntry> {
    &log_channel().0
}

pub fn log_rx() -> &'static async_channel::Receiver<LogEntry> {
    &log_channel().1
}

pub fn now_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn format_message(msg: &Message) -> String {
    match msg {
        Message::Connected => "Connected".to_string(),
        Message::Disconnected => "Disconnected".to_string(),
        Message::Next => "Next".to_string(),
        Message::Judgment(j) => j.as_str(),
    }
}
