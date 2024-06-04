use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio::sync::mpsc;

use crate::{
  configuration::app_configuration::Config,
  tui::{
    self,
    action::Action,
    components::{
      cpu::Cpu, disks::DiskTable, fps::FpsCounter, home::Home, network::NetworkComponent, process_table::ProcessTable,
      Component,
    },
    mode::Mode,
    ui::{Event, Tui},
  },
};

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let home = Home::new();
    let cpu = Cpu::new();
    let process_table = ProcessTable::new();
    let disk_table = DiskTable::new();
    let network_component = NetworkComponent::new();

    let config = Config::new()?;
    let mode = Mode::Home;
    Ok(App {
      tick_rate,
      frame_rate,
      components: vec![Box::new(cpu), Box::new(process_table), Box::new(disk_table), Box::new(network_component)],
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut tui = Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
    // tui.mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
    }

    loop {
      if let Some(e) = tui.next().await {
        match e {
          Event::Quit => action_tx.send(Action::Quit)?,
          Event::Tick => action_tx.send(Action::Tick)?,
          Event::Render => action_tx.send(Action::Render)?,
          Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          Event::Key(key) => {
            if let Some(keymap) = self.config.keybindings.get(&self.mode) {
              if let Some(action) = keymap.get(&vec![key]) {
                log::info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
              } else {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                }
              }
            };
          },
          Event::DataUpdate(ref data) => action_tx.send(Action::DataUpdate(data.clone()))?,
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.handle_events(Some(e.clone()))? {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          // log::debug!("{action:?}");
        }
        match action {
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          // Since we do not need to currently do anything extra with the data we can let the loop below handle sending the action to each component.
          // Action::DataUpdate(boxed_data) => {},
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(Action::Resume)?;
        tui = Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
