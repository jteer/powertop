use std::{collections::VecDeque, time::Instant};

use color_eyre::{
  eyre::{Ok, Result},
  owo_colors::OwoColorize,
};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};

use super::Component;
use crate::{
  configuration::app_configuration::Config,
  data_services::network::{NetworkData, NetworkDataCollection},
  tui::{action::Action, ui::Frame},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkViewModel {
  received: VecDeque<u64>,
  transmitted: VecDeque<u64>,
  total_transmitted: u64,
  total_received: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkComponent {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: NetworkDataCollection,
  network_view_model: NetworkViewModel,
}

impl Default for NetworkComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl NetworkComponent {
  pub fn new() -> Self {
    Self {
      app_start_time: Instant::now(),
      render_start_time: Instant::now(),
      collected_data: [].to_vec(),
      network_view_model: NetworkViewModel {
        received: VecDeque::with_capacity(25),
        transmitted: VecDeque::with_capacity(25),
        total_transmitted: 0,
        total_received: 0,
      },
    }
  }

  fn update_data_stats(&mut self, new_data: NetworkDataCollection) {
    log::debug!("Updating Network Component with new data: {:?}", new_data.len());
    // self.collected_data.append(&mut new_data);

    let received = new_data.iter().map(|c| c.received).collect_vec();
    if self.network_view_model.received.len() == 25 {
      self.network_view_model.received.pop_front();
    }

    self.network_view_model.received.push_back(received.iter().sum());

    let transmitted = new_data.iter().map(|c| c.transmitted).collect_vec();
    if self.network_view_model.transmitted.len() == 25 {
      self.network_view_model.transmitted.pop_front();
    }

    self.network_view_model.transmitted.push_back(transmitted.iter().sum());

    self.network_view_model.total_transmitted = new_data.iter().map(|c| c.total_transmitted).sum();
    self.network_view_model.total_received = new_data.iter().map(|c| c.total_received).sum();
  }
}

impl NetworkData {
}

impl Component for NetworkComponent {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::DataUpdate(data) = action {
      match data.networks {
        Some(d) => self.update_data_stats(d),
        None => {
          log::debug!("Received Action with no data.")
        },
      }
    }
    Ok(None)
  }

  fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![
        Constraint::Percentage(100), // Top row spans whole width
        Constraint::Percentage(50),  // Bottom row split 50/50
      ])
      .split(area);

    let bottom_row_rects = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![
        Constraint::Percentage(33), // Left half
        Constraint::Percentage(33), // Right half
        Constraint::Percentage(33), // Right half
      ])
      .split(rects[1]);

    let network_area = bottom_row_rects[2];

    let outer_block = Block::bordered().title("Network");
    let inner = outer_block.inner(network_area);
    let inner_split = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(inner);

    // TODO Value Scaling and Units
    let continuous_rx_values = self.network_view_model.received.make_contiguous();
    let rx_title = format!("Received {}", self.network_view_model.total_received);
    let rx_spark = Sparkline::default()
      .block(Block::new().title(rx_title))
      .data(&continuous_rx_values)
      .max(15)
      .direction(RenderDirection::LeftToRight)
      .style(Style::default().red().black());

    let continuous_tx_values = self.network_view_model.transmitted.make_contiguous();
    let tx_title = format!("Transmitted {}", self.network_view_model.total_transmitted);
    let tx_spark = Sparkline::default()
      .block(Block::new().title(tx_title))
      .data(&continuous_tx_values)
      .max(15)
      .direction(RenderDirection::LeftToRight)
      .style(Style::default().red().black());

    frame.render_widget(outer_block, network_area);
    frame.render_widget(rx_spark, inner_split[0]);
    frame.render_widget(tx_spark.clone(), inner_split[1]);

    Ok(())
  }
}
