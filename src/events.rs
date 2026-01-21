use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    FileChanged,
    SyncFinished(Result<String, String>),
}

pub struct EventHandler {
    pub sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(tick_rate_ms);

        let tx_input = tx.clone();
        thread::spawn(move || {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Event::Key(key) = event::read().unwrap() {
                        if tx_input.send(AppEvent::Input(key)).is_err() {
                            break;
                        }
                    }
                }
                if tx_input.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self {
            sender: tx,
            receiver: rx,
        }
    }

    pub fn next(&self) -> Result<AppEvent> {
        Ok(self.receiver.recv()?)
    }
}
