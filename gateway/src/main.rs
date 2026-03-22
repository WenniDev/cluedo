#![windows_subsystem = "windows"]

use std::net::{TcpListener, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::prelude::*;
use gpui::*;

use socket::Message;

const DEFAULT_UDP_ADDR: &str = "127.0.0.1:7877";
const WS_ADDR: &str = "127.0.0.1:7878";

type WsClients = Arc<Mutex<Vec<mpsc::Sender<Vec<u8>>>>>;

struct LogEntry {
    time: String,
    msg: String,
}

fn now_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn format_message(msg: &Message) -> String {
    match msg {
        Message::Connected => "Connected".to_string(),
        Message::Disconnected => "Disconnected".to_string(),
        Message::Next => "Next".to_string(),
        Message::Judgment(j) => j.as_str(),
    }
}

fn start_udp_thread(
    addr: String,
    ws_clients: WsClients,
    log_tx: async_channel::Sender<LogEntry>,
    stop: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let sock = match UdpSocket::bind(&addr) {
            Ok(s) => s,
            Err(e) => {
                log_tx
                    .try_send(LogEntry {
                        time: now_str(),
                        msg: format!("Error binding {addr}: {e}"),
                    })
                    .ok();
                return;
            }
        };
        sock.set_read_timeout(Some(Duration::from_millis(100))).ok();
        log_tx
            .try_send(LogEntry {
                time: now_str(),
                msg: format!("Listening on {addr}"),
            })
            .ok();

        let mut buf = [0u8; 4096];
        while !stop.load(Ordering::Relaxed) {
            match sock.recv(&mut buf) {
                Ok(len) => {
                    let data = buf[..len].to_vec();
                    if let Some(msg) = socket::decode(&data) {
                        log_tx
                            .try_send(LogEntry {
                                time: now_str(),
                                msg: format_message(&msg),
                            })
                            .ok();
                    }
                    let mut clients = ws_clients.lock().unwrap();
                    clients.retain(|tx| tx.send(data.clone()).is_ok());
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => {
                    log_tx
                        .try_send(LogEntry {
                            time: now_str(),
                            msg: format!("UDP error: {e}"),
                        })
                        .ok();
                    break;
                }
            }
        }
    });
}

struct AppState {
    log: Vec<LogEntry>,
}

struct OverlayView {
    state: Entity<AppState>,
    ws_clients: WsClients,
    log_tx: async_channel::Sender<LogEntry>,
    input_text: String,
    udp_stop: Arc<AtomicBool>,
    focus: FocusHandle,
}

impl OverlayView {
    fn new(
        state: Entity<AppState>,
        ws_clients: WsClients,
        log_tx: async_channel::Sender<LogEntry>,
        udp_stop: Arc<AtomicBool>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus);
        Self {
            state,
            ws_clients,
            log_tx,
            input_text: DEFAULT_UDP_ADDR.to_string(),
            udp_stop,
            focus,
        }
    }

    fn connect(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.udp_stop.store(true, Ordering::Relaxed);
        let stop = Arc::new(AtomicBool::new(false));
        self.udp_stop = stop.clone();
        self.state.update(cx, |s, cx| {
            s.log.clear();
            cx.notify();
        });
        start_udp_thread(
            self.input_text.clone(),
            self.ws_clients.clone(),
            self.log_tx.clone(),
            stop,
        );
    }
}

impl Focusable for OverlayView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for OverlayView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let log_entries: Vec<(String, String)> = {
            let state = self.state.read(cx);
            state
                .log
                .iter()
                .rev()
                .take(200)
                .map(|e| (e.time.clone(), e.msg.clone()))
                .collect()
        };
        let input_text = self.input_text.clone();

        div()
            .key_context("OverlayView")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let key = &event.keystroke.key;
                match key.as_str() {
                    "backspace" => {
                        this.input_text.pop();
                        cx.notify();
                    }
                    "return" => {
                        this.connect(window, cx);
                    }
                    k if k.len() == 1 && !event.keystroke.modifiers.platform => {
                        this.input_text.push_str(k);
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .size_full()
            .flex()
            .flex_col()
            .p_4()
            .gap_4()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xd4d4d4))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .items_center()
                    .child(div().child("UDP:"))
                    .child(
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(rgb(0x555555))
                            .rounded_sm()
                            .px_2()
                            .py_1()
                            .bg(rgb(0x2d2d2d))
                            .child(input_text),
                    )
                    .child(
                        div()
                            .border_1()
                            .border_color(rgb(0x0078d4))
                            .rounded_sm()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x0078d4))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _, window, cx| this.connect(window, cx)),
                            )
                            .child("Connect"),
                    ),
            )
            .child(
                div()
                    .id("log")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .overflow_y_scroll()
                    .gap_1()
                    .children(log_entries.into_iter().enumerate().map(|(i, (time, msg))| {
                        div()
                            .id(i)
                            .px_1()
                            .rounded_sm()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .hover(|s| s.bg(rgb(0x2d2d2d)))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener({
                                    let line = format!("[{time}] {msg}");
                                    move |_this, _, _window, cx| {
                                        cx.write_to_clipboard(ClipboardItem::new_string(
                                            line.clone(),
                                        ));
                                    }
                                }),
                            )
                            .child(div().text_color(rgb(0x888888)).child(time))
                            .child(div().child(msg))
                    })),
            )
    }
}

fn main() {
    let (log_tx, log_rx) = async_channel::unbounded::<LogEntry>();
    let ws_clients: WsClients = Arc::new(Mutex::new(Vec::new()));

    // WebSocket server thread
    let ws_clients_ws = ws_clients.clone();
    thread::spawn(move || {
        let listener = TcpListener::bind(WS_ADDR).expect("failed to bind WebSocket");
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let Ok(mut ws) = tungstenite::accept(stream) else {
                continue;
            };
            let (tx, rx) = mpsc::channel::<Vec<u8>>();
            ws_clients_ws.lock().unwrap().push(tx);
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
    });

    // Initial UDP listener
    let initial_stop = Arc::new(AtomicBool::new(false));
    start_udp_thread(
        DEFAULT_UDP_ADDR.to_string(),
        ws_clients.clone(),
        log_tx.clone(),
        initial_stop.clone(),
    );

    Application::new().run(|cx: &mut App| {
        let state = cx.new(|_cx| AppState { log: Vec::new() });

        // Background task: pump log messages into the model
        let state_handle = state.clone();
        cx.spawn(async move |cx: &mut AsyncApp| {
            while let Ok(entry) = log_rx.recv().await {
                cx.update_entity(&state_handle, |s, cx| {
                    s.log.push(entry);
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();

        let _ = cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("DDR Overlay".into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::centered(size(px(480.), px(320.)), cx)),
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| OverlayView::new(state, ws_clients, log_tx, initial_stop, window, cx))
            },
        );
    });
}
