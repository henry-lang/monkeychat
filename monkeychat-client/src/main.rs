#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod connection;
mod message;

use eframe::{
    self,
    egui::{
        widgets::Button, CentralPanel, Color32, Context, Key, RichText, TopBottomPanel, Visuals,
        Window,
    },
    App, Frame, NativeOptions,
};

use message::{ToConnThread, ToMainThread};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc::{self, Receiver, Sender};

enum State {
    NotConnected {
        server: String,
        connecting: bool,
        err_message: Option<String>,
    },
    Connected {
        message: String,
        messages: Vec<String>,
    },
}

struct ConnThread {
    handle: JoinHandle<()>,
    tx: Sender<ToConnThread>,
    rx: Receiver<ToMainThread>,
}

struct Client {
    state: State,
    conn_thread: ConnThread,
}

impl Client {
    pub fn new() -> Self {
        let (main_tx, main_rx) = mpsc::channel(32);
        let (conn_tx, conn_rx) = mpsc::channel(32);
        let handle = thread::spawn(|| connection::conn_thread(main_tx, conn_rx));
        Self {
            conn_thread: ConnThread {
                handle,
                tx: conn_tx,
                rx: main_rx,
            },
            state: State::NotConnected {
                server: String::new(),
                connecting: false,
                err_message: None,
            },
        }
    }

    pub fn handle_messages(&mut self) {
        while let Ok(msg) = self.conn_thread.rx.try_recv() {
            println!("{:?}", msg);
            use message::ToMainThread::*;

            match msg {
                ConnectionError(error) => {
                    if let State::NotConnected {
                        ref mut err_message,
                        ref mut connecting,
                        ..
                    } = self.state
                    {
                        *err_message = Some(error);
                        *connecting = false;
                    }
                }

                _ => (),
            }
        }
    }
}

impl App for Client {
    fn update(&mut self, ctx: &Context, _: &mut Frame) {
        self.handle_messages();

        match self.state {
            State::NotConnected {
                ref mut server,
                ref mut connecting,
                ref err_message,
            } => {
                let mut connect = false;
                Window::new("config").show(ctx, |ui| {
                    if let Some(msg) = err_message {
                        ui.label(RichText::new(msg).color(Color32::RED));
                    }
                    ui.label("server address");
                    if ui.text_edit_singleline(server).lost_focus()
                        && ui.input().key_pressed(Key::Enter)
                        || ui
                            .add_enabled(!*connecting, Button::new("connect"))
                            .clicked()
                    {
                        connect = true;
                    }
                });

                if connect {
                    let addr = server.parse();

                    match addr {
                        Ok(addr) => {
                            self.conn_thread
                                .tx
                                .blocking_send(ToConnThread::Connect(addr))
                                .unwrap();

                            *connecting = true;
                        }

                        Err(_) => {
                            println!("Invalid socket address.");
                        }
                    }
                }
            }

            State::Connected {
                ref mut message,
                ref messages,
            } => {
                TopBottomPanel::bottom("entry").show(ctx, |ui| {
                    let text_edit = ui.text_edit_singleline(message);
                    if text_edit.lost_focus() && ui.input().key_pressed(Key::Enter) {
                        self.conn_thread
                            .tx
                            .blocking_send(ToConnThread::SendMessage(message.clone()))
                            .unwrap();

                        message.clear();
                    }
                });
                CentralPanel::default().show(ctx, |ui| {
                    for message in messages {
                        ui.label(message);
                    }
                });
            }
        }
    }

    fn on_exit(&mut self, _: &eframe::glow::Context) {
        self.conn_thread
            .tx
            .blocking_send(ToConnThread::Shutdown)
            .unwrap();
    }
}

fn main() {
    eframe::run_native(
        "monkeychat client",
        NativeOptions::default(),
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(Visuals::dark());
            Box::new(Client::new())
        }),
    );
}
