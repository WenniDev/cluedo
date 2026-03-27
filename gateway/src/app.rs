use std::io;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::log::log_rx;
use crate::network::{
    COUNT_FAST, COUNT_GOOD, COUNT_GREAT, COUNT_MARVELOUS, COUNT_MISS, COUNT_OK, COUNT_PERFECT,
    COUNT_SLOW, SONGS_PLAYED, STOP_UDP, STOP_WS, WsClients,
    start_udp_thread, start_ws_thread,
};
use crate::ui;

pub struct Counts {
    pub marvelous: u32,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub ok: u32,
    pub miss: u32,
    pub fast: u32,
    pub slow: u32,
}

pub struct Stats {
    pub songs: u32,
    pub elapsed_secs: u64,
}

pub enum Mode {
    Normal,
    Settings {
        ddr: String,
        obs: String,
        field: usize,
    },
}

pub fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ws_clients: &WsClients,
) -> io::Result<()> {
    let mut ddr_addr = crate::DEFAULT_DDR_ADDR.to_string();
    let mut obs_addr = crate::DEFAULT_OBS_ADDR.to_string();
    let mut log: Vec<(String, String)> = Vec::new();
    let mut mode = Mode::Normal;
    let start_time = Instant::now();

    loop {
        while let Ok(entry) = log_rx().try_recv() {
            log.push((entry.time, entry.msg));
        }
        if log.len() > 500 {
            log.drain(0..log.len() - 500);
        }

        let counts = Counts {
            marvelous: COUNT_MARVELOUS.load(Ordering::Relaxed),
            perfect: COUNT_PERFECT.load(Ordering::Relaxed),
            great: COUNT_GREAT.load(Ordering::Relaxed),
            good: COUNT_GOOD.load(Ordering::Relaxed),
            ok: COUNT_OK.load(Ordering::Relaxed),
            miss: COUNT_MISS.load(Ordering::Relaxed),
            fast: COUNT_FAST.load(Ordering::Relaxed),
            slow: COUNT_SLOW.load(Ordering::Relaxed),
        };
        let stats = Stats {
            songs: SONGS_PLAYED.load(Ordering::Relaxed),
            elapsed_secs: start_time.elapsed().as_secs(),
        };
        terminal.draw(|f| ui::draw(f, &ddr_addr, &obs_addr, &counts, &stats, &log, &mode))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match &mut mode {
                    Mode::Normal => match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Char('q') if key.modifiers.is_empty() => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            break;
                        }
                        KeyCode::Char('s') if key.modifiers.is_empty() => {
                            mode = Mode::Settings {
                                ddr: ddr_addr.clone(),
                                obs: obs_addr.clone(),
                                field: 0,
                            };
                        }
                        KeyCode::Enter => do_connect(&ddr_addr, &obs_addr, ws_clients, &mut log),
                        KeyCode::Char(c) => ddr_addr.push(c),
                        KeyCode::Backspace => {
                            ddr_addr.pop();
                        }
                        _ => {}
                    },
                    Mode::Settings { ddr, obs, field } => match key.code {
                        KeyCode::Esc => mode = Mode::Normal,
                        KeyCode::Tab => *field = 1 - *field,
                        KeyCode::Enter => {
                            ddr_addr = ddr.clone();
                            obs_addr = obs.clone();
                            mode = Mode::Normal;
                        }
                        KeyCode::Char(c) => {
                            if *field == 0 {
                                ddr.push(c);
                            } else {
                                obs.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if *field == 0 {
                                ddr.pop();
                            } else {
                                obs.pop();
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

pub fn do_connect(ddr_addr: &str, obs_addr: &str, ws_clients: &WsClients, log: &mut Vec<(String, String)>) {
    STOP_UDP.store(true, Ordering::Relaxed);
    STOP_WS.store(true, Ordering::Relaxed);
    log.clear();
    start_udp_thread(ddr_addr.to_string(), ws_clients.clone());
    start_ws_thread(obs_addr.to_string(), ws_clients.clone());
}
