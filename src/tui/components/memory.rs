use std::{collections::VecDeque, fmt, time::Instant};

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
  data_services::memory::MemoryData,
  tui::{action::Action, ui::Frame},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryViewModel {
  available_ram: VecDeque<(f64, f64)>,
  available_swap: VecDeque<(f64, f64)>,
  total_ram: u64,
  total_swap: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryComponent {
  app_start_time: Instant,
  render_start_time: Instant,
  memory_view_model: MemoryViewModel,
}

impl Default for MemoryComponent {
  fn default() -> Self {
    Self::new()
  }
}

impl MemoryComponent {
  pub const WINDOW_SIZE: usize = 10;

  pub fn new() -> Self {
    Self {
      app_start_time: Instant::now(),
      render_start_time: Instant::now(),
      memory_view_model: MemoryViewModel {
        total_ram: 0,
        total_swap: 0,
        available_ram: VecDeque::with_capacity(Self::WINDOW_SIZE),
        available_swap: VecDeque::with_capacity(Self::WINDOW_SIZE),
      },
    }
  }

  fn update_data_stats(&mut self, new_data: MemoryData) {
    log::debug!("Updating Memory Component with new data: {:?}", new_data);

    self.memory_view_model.total_ram = new_data.total_ram;
    self.memory_view_model.total_swap = new_data.total_swap;

    let (ram_percent, swap_percent) = new_data.usage_percentages();
    if self.memory_view_model.available_ram.len() == MemoryComponent::WINDOW_SIZE {
      self.memory_view_model.available_ram.pop_front();
      //   Shift the x value by -1 so the graph plots correctly
      self.memory_view_model.available_ram.iter_mut().for_each(|f| f.0 -= 1.0);
    }
    self.memory_view_model.available_ram.push_back((self.memory_view_model.available_ram.len() as f64, ram_percent));

    if self.memory_view_model.available_swap.len() == MemoryComponent::WINDOW_SIZE {
      self.memory_view_model.available_swap.pop_front();
      //   Shift the x value by -1 so the graph plots correctly
      self.memory_view_model.available_swap.iter_mut().for_each(|f| f.0 -= 1.0);
    }
    self.memory_view_model.available_swap.push_back((self.memory_view_model.available_swap.len() as f64, swap_percent));
  }
}

impl Component for MemoryComponent {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::DataUpdate(data) = action {
      match data.memory {
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
      .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(area);
    let top_row = rects[0];
    let top_row_rects = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(top_row);
    let memory_rect = top_row_rects[1];

    let x_axis = Axis::default().style(Style::default().white()).bounds([0.0, 100.0]);
    let y_axis = Axis::default().style(Style::default().white()).bounds([0.0, 100.0]);

    let ram_data = self.memory_view_model.available_ram.make_contiguous();
    let swap_data = self.memory_view_model.available_swap.make_contiguous();

    let zero_percent = String::from("0.0 %");
    let current_ram_value_str =
      if let Some(v) = ram_data.last() { format!("{:.1$}%", v.1, 2) } else { zero_percent.clone() };
    let current_swap_value_str = if let Some(v) = swap_data.last() { format!("{:.1$}%", v.1, 2) } else { zero_percent };

    let ram_data_set = Dataset::default()
      .name(format!("RAM {}", current_ram_value_str))
      .marker(symbols::Marker::Dot)
      .graph_type(GraphType::Line)
      .style(Style::default().cyan())
      .data(ram_data);

    let swap_data_set = Dataset::default()
      .name(format!("SWAP {}", current_swap_value_str))
      .marker(symbols::Marker::Dot)
      .graph_type(GraphType::Line)
      .style(Style::default().red())
      .data(swap_data);

    let chart = Chart::new(vec![ram_data_set, swap_data_set])
      .block(Block::bordered().title("Memory"))
      .x_axis(x_axis)
      .y_axis(y_axis)
      .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)))
      .legend_position(Some(LegendPosition::TopRight));

    frame.render_widget(chart, memory_rect);

    Ok(())
  }
}
