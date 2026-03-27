mod app;
mod log;
mod network;
mod ui;

use std::io;
use std::sync::Arc;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use network::WsClients;

const DEFAULT_DDR_ADDR: &str = "127.0.0.1:7877";
const DEFAULT_OBS_ADDR: &str = "127.0.0.1:7878";

fn main() -> io::Result<()> {
    let ws_clients: WsClients = Arc::new(std::sync::Mutex::new(Vec::new()));

    network::start_udp_thread(DEFAULT_DDR_ADDR.to_string(), ws_clients.clone());
    network::start_ws_thread(DEFAULT_OBS_ADDR.to_string(), ws_clients.clone());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = app::run(&mut terminal, &ws_clients);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
