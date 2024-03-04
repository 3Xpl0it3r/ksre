use color_eyre::Result;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::{
    cursor,
    event::{Event as CrosstermEvent, KeyEventKind},
    terminal::EnterAlternateScreen,
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use std::ops::{Deref, DerefMut};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::event::Event;

const DEFAULT_FRAME_RATE: f64 = 60.0;

pub const DEBUG: bool = false;

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stderr>>,
    pub ptr_long_job: JoinHandle<()>,
    pub ptr_cancell: CancellationToken,
    pub rx_event: Receiver<Event>,
    pub tx_event: Sender<Event>,
    pub nr_frame_rate: f64,
}

// Default[#TODO] (should add some comments)
impl Default for Tui {
    fn default() -> Self {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stderr())).unwrap();
        let (tx_event, rx_event) = mpsc::channel(100);
        let task = tokio::spawn(async {});
        let frame_rate = DEFAULT_FRAME_RATE;
        let cancellation_token = CancellationToken::new();
        Self {
            terminal,
            ptr_long_job: task,
            ptr_cancell: cancellation_token,
            rx_event,
            tx_event,
            nr_frame_rate: frame_rate,
        }
    }
}

// Tui[#TODO] (should add some comments)
impl Tui {
    pub fn new() -> Result<Self> {
        let mut tui = Self::default();
        tui.run().unwrap();
        Ok(tui)
    }

    pub fn run(&mut self) -> Result<()> {
        initialize_panic_handler();

        let mut second = 1.0;
        if DEBUG {
            second = 60.0
        } else {
            self.enter()?;
        }

        let frame_delay = std::time::Duration::from_secs_f64(second / self.nr_frame_rate);
        let _tx_event = self.tx_event.clone();
        let _cancellation_token = self.ptr_cancell.clone();

        self.ptr_long_job = tokio::spawn(async move {
            let mut event_reader = crossterm::event::EventStream::new();
            let mut frame_interval = tokio::time::interval(frame_delay);
            loop {
                let frame_tick = frame_interval.tick();
                let crossterm_event = event_reader.next().fuse();
                tokio::select! {
                    _ = _cancellation_token.cancelled() => break,
                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(event)) => {
                                if let CrosstermEvent::Key(key) = event {
                                    if key.kind == KeyEventKind::Press {
                                        _tx_event.send(Event::Key(key.into())).await.unwrap();
                                    }
                                }
                            },
                            Some(Err(_)) => {_tx_event.send(Event::Error).await.unwrap();},
                            None => {_tx_event.send(Event::Error).await.unwrap();}
                        }
                    },
                    _ = frame_tick => _tx_event.send(Event::Tick).await.unwrap(),
                }
            }
        });

        Ok(())
    }

    fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), EnterAlternateScreen, cursor::Hide)?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        if !self.ptr_cancell.is_cancelled() {
            self.ptr_cancell.cancel();
        }
        if crossterm::terminal::is_raw_mode_enabled()? {
            crossterm::terminal::disable_raw_mode()?;
        }
        if !DEBUG {
            crossterm::execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show)?;
        }

        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx_event.recv().await
    }
}

impl Deref for Tui {
    type Target = ratatui::Terminal<Backend<std::io::Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Tui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

// Drop[#TODO] (should add some comments)
impl Drop for Tui {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

fn initialize_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}
