use std::{
  ops::{Deref, DerefMut},
  time::Duration,
};

use color_eyre::eyre::{eyre, Result};
use crossterm::{
  cursor,
  event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event as CrosstermEvent,
    KeyEvent, KeyEventKind, MouseEvent,
  },
  terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use serde::{Deserialize, Serialize};
use tokio::{
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::data_services::data_collector::{DataCollected, DataCollector, SysinfoSource};

pub type IO = std::io::Stdout;
pub fn io() -> IO {
  std::io::stdout()
}
pub type Frame<'a> = ratatui::Frame<'a>;

#[derive(Clone, Debug)]
pub enum Event {
  Init,
  Quit,
  Error,
  Closed,
  Tick,
  Render,
  FocusGained,
  FocusLost,
  Paste(String),
  Key(KeyEvent),
  Mouse(MouseEvent),
  Resize(u16, u16),
  DataUpdate(Box<DataCollected>),
}

/// Interval to sleep between task status checks.
const SLEEP_INTERVAL: Duration = Duration::from_millis(1);

/// Maximum number of retries for checking task status.
const MAX_RETRIES: usize = 10;

/// Interval to sleep between data collection task updates.
const DATA_COLLECTION_SLEEP_INTERVAL: Duration = Duration::from_millis(1000);

pub struct Tui {
  pub terminal: ratatui::Terminal<Backend<IO>>,
  pub task: JoinHandle<()>,
  pub data_collection_task: JoinHandle<()>,

  pub cancellation_token: CancellationToken,
  pub event_rx: UnboundedReceiver<Event>,
  pub event_tx: UnboundedSender<Event>,
  pub frame_rate: f64,
  pub tick_rate: f64,
  pub mouse: bool,
  pub paste: bool,
}

impl Tui {
  pub fn new() -> Result<Self> {
    let tick_rate = 4.0;
    let frame_rate = 60.0;
    let terminal = ratatui::Terminal::new(Backend::new(io()))?;
    let (event_tx, event_rx) = mpsc::unbounded_channel();
    let cancellation_token = CancellationToken::new();
    let task = tokio::spawn(async {});
    let data_collection_task = tokio::spawn(async {});
    let mouse = false;
    let paste = false;
    Ok(Self {
      terminal,
      task,
      data_collection_task,
      cancellation_token,
      event_rx,
      event_tx,
      frame_rate,
      tick_rate,
      mouse,
      paste,
    })
  }

  pub fn tick_rate(mut self, tick_rate: f64) -> Self {
    self.tick_rate = tick_rate;
    self
  }

  pub fn frame_rate(mut self, frame_rate: f64) -> Self {
    self.frame_rate = frame_rate;
    self
  }

  pub fn mouse(mut self, mouse: bool) -> Self {
    self.mouse = mouse;
    self
  }

  pub fn paste(mut self, paste: bool) -> Self {
    self.paste = paste;
    self
  }

  pub fn start(&mut self) {
    self.cancel();
    self.cancellation_token = CancellationToken::new();

    // Spawn a task for data collection
    self.spawn_data_collection_task();

    // Spawn a task for the main(input) event loop
    self.spawn_input_event_loop_task();
  }

  fn spawn_input_event_loop_task(&mut self) {
    let tick_delay = std::time::Duration::from_secs_f64(1.0 / self.tick_rate);
    let render_delay = std::time::Duration::from_secs_f64(1.0 / self.frame_rate);
    let _cancellation_token = self.cancellation_token.clone();
    let _event_tx = self.event_tx.clone();
    self.task = tokio::spawn(async move {
      let mut reader = crossterm::event::EventStream::new();
      let mut tick_interval = tokio::time::interval(tick_delay);
      let mut render_interval = tokio::time::interval(render_delay);
      _event_tx.send(Event::Init).unwrap();
      loop {
        let tick_delay = tick_interval.tick();
        let render_delay = render_interval.tick();
        let crossterm_event = reader.next().fuse();

        tokio::select! {
          _ = _cancellation_token.cancelled() => {
            break;
          }
          maybe_event = crossterm_event => {
            match maybe_event {
              Some(Ok(evt)) => {
                match evt {
                  CrosstermEvent::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                      _event_tx.send(Event::Key(key)).unwrap();
                    }
                  },
                  CrosstermEvent::Mouse(mouse) => {
                    _event_tx.send(Event::Mouse(mouse)).unwrap();
                  },
                  CrosstermEvent::Resize(x, y) => {
                    _event_tx.send(Event::Resize(x, y)).unwrap();
                  },
                  CrosstermEvent::FocusLost => {
                    _event_tx.send(Event::FocusLost).unwrap();
                  },
                  CrosstermEvent::FocusGained => {
                    _event_tx.send(Event::FocusGained).unwrap();
                  },
                  CrosstermEvent::Paste(s) => {
                    _event_tx.send(Event::Paste(s)).unwrap();
                  },
                }
              }
              Some(Err(_)) => {
                _event_tx.send(Event::Error).unwrap();
              }
              None => {},
            }
          },
          _ = tick_delay => {
              _event_tx.send(Event::Tick).unwrap();
          },
          _ = render_delay => {
              _event_tx.send(Event::Render).unwrap();
          },
        }
      }
    });
  }

  fn spawn_data_collection_task(&mut self) {
    let data_event_tx = self.event_tx.clone();
    let data_collection_token = self.cancellation_token.clone();
    self.data_collection_task = tokio::spawn(async move {
      let mut data_state: DataCollector = DataCollector::new();

      loop {
        // Check for cancellation
        if data_collection_token.is_cancelled() {
          break;
        }

        data_state.update_data();
        let event = Event::DataUpdate(Box::from(data_state.data));

        data_state.data = DataCollected::default();
        if data_event_tx.send(event).is_err() {
          break;
        }

        // Add a delay to prevent CPU monopolization
        // TODO Make delay configurable
        tokio::time::sleep(DATA_COLLECTION_SLEEP_INTERVAL).await;
      }
    });
  }

  /// Stops the TUI by canceling and aborting ongoing tasks.
  ///
  /// This function will first cancel the main and data collection tasks,
  /// then attempt to abort them if they do not finish within a reasonable time.
  ///
  /// # Errors
  ///
  /// Returns an error if either task fails to abort within the specified timeout.
  pub fn stop(&self) -> Result<()> {
    self.cancel();

    // Abort the main task
    self.abort_task(&self.task, MAX_RETRIES)?;

    // Abort the data collection task
    self.abort_task(&self.data_collection_task, MAX_RETRIES)?;

    Ok(())
  }

  /// Attempts to abort a given task, waiting for it to finish.
  ///
  /// This function will check if the task is finished, and if not, will
  /// attempt to abort it after a certain number of retries. It will wait
  /// for a short interval between each check.
  ///
  /// # Parameters
  ///
  /// - `task`: The task to be aborted.
  /// - `max_retries`: The maximum number of retries before forcing the task to abort.
  ///
  /// # Errors
  ///
  /// Returns an error if the task fails to abort within the specified number of retries.
  fn abort_task(&self, task: &JoinHandle<()>, max_retries: usize) -> Result<()> {
    for attempt in 0..max_retries {
      if task.is_finished() {
        return Ok(());
      }

      if attempt == max_retries / 2 {
        task.abort();
      }

      std::thread::sleep(SLEEP_INTERVAL);
    }

    Err(eyre!("Failed to abort task within {} milliseconds", max_retries * SLEEP_INTERVAL.as_millis() as usize))
  }

  pub fn enter(&mut self) -> Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io(), EnterAlternateScreen, cursor::Hide)?;
    if self.mouse {
      crossterm::execute!(io(), EnableMouseCapture)?;
    }
    if self.paste {
      crossterm::execute!(io(), EnableBracketedPaste)?;
    }
    self.start();
    Ok(())
  }

  pub fn exit(&mut self) -> Result<()> {
    self.stop()?;
    if crossterm::terminal::is_raw_mode_enabled()? {
      self.flush()?;
      if self.paste {
        crossterm::execute!(io(), DisableBracketedPaste)?;
      }
      if self.mouse {
        crossterm::execute!(io(), DisableMouseCapture)?;
      }
      crossterm::execute!(io(), LeaveAlternateScreen, cursor::Show)?;
      crossterm::terminal::disable_raw_mode()?;
    }
    Ok(())
  }

  pub fn cancel(&self) {
    self.cancellation_token.cancel();
  }

  pub fn suspend(&mut self) -> Result<()> {
    self.exit()?;
    #[cfg(not(windows))]
    signal_hook::low_level::raise(signal_hook::consts::signal::SIGTSTP)?;
    Ok(())
  }

  pub fn resume(&mut self) -> Result<()> {
    self.enter()?;
    Ok(())
  }

  pub async fn next(&mut self) -> Option<Event> {
    self.event_rx.recv().await
  }
}

impl Deref for Tui {
  type Target = ratatui::Terminal<Backend<IO>>;

  fn deref(&self) -> &Self::Target {
    &self.terminal
  }
}

impl DerefMut for Tui {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.terminal
  }
}

impl Drop for Tui {
  fn drop(&mut self) {
    self.exit().unwrap();
  }
}
