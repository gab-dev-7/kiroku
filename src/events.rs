use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    FileChanged,
}

// handles keyboard inputs and tick events in a separate thread
pub struct EventHandler {
    pub sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    paused: Arc<AtomicBool>,
}

impl EventHandler {
    // spawn a background thread to poll for input events
    pub fn new(tick_rate_ms: u64) -> Self {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let paused = Arc::new(AtomicBool::new(false));

        let tx_input = tx.clone();
        let thread_paused = paused.clone();

        thread::spawn(move || {
            loop {
                if thread_paused.load(Ordering::SeqCst) {
                    thread::sleep(tick_rate);
                    continue;
                }

                if event::poll(tick_rate).unwrap_or(false) {
                    if let Event::Key(key) = event::read().unwrap()
                        && tx_input.send(AppEvent::Input(key)).is_err()
                    {
                        break;
                    }
                } else if tx_input.send(AppEvent::Tick).is_err() {
                    break;
                }
            }
        });

        Self {
            sender: tx,
            receiver: rx,
            paused,
        }
    }

    pub fn next(&self) -> Result<AppEvent> {
        Ok(self.receiver.recv()?)
    }

    // pause input polling
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    // resume input polling
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }
}
