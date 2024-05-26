use std::time::Instant;

use color_eyre::eyre::Result;
use itertools::Itertools;
use libc::group;
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{action::Action, data_services::cpu::CpuDataCollection, tui::Frame};

#[derive(Debug, Clone, PartialEq)]
pub struct Cpu {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: CpuDataCollection,
  max_usage: f64,
}

impl Default for Cpu {
  fn default() -> Self {
    Self::new()
  }
}

impl Cpu {
  pub fn new() -> Self {
    Self {
      app_start_time: Instant::now(),
      render_start_time: Instant::now(),
      collected_data: [].to_vec(),
      max_usage: 0.0,
    }
  }

  fn fetch_data(&mut self) -> Result<()> {
    let now = Instant::now();
    let elapsed = (now - self.app_start_time).as_secs_f64();
    if elapsed >= 1.0 {
      self.app_start_time = now;
    }
    Ok(())
  }

  fn get_cpu_datasets(&self) -> Vec<Dataset> {
    let colors = vec![
      Style::default().cyan(),
      Style::default().magenta(),
      Style::default().yellow(),
      Style::default().green(),
      Style::default().blue(),
      Style::default().red(),
    ];

    let mut color_iter = colors.iter().cycle();

    // Map the collected data to a vec of Dataset where each Dataset is mapped to each cpu
    let datasets = self
      .collected_data
      .iter()
      .enumerate()
      .chunk_by(|(_, cpu_data)| &cpu_data.cpu_name)
      .into_iter()
      .map(|f| {
        let color = color_iter.next().unwrap();
        let (cpu_name, group_data) = f;

        Dataset::default()
          .name(cpu_name.to_string())
          .marker(symbols::Marker::Dot)
          .graph_type(GraphType::Scatter)
          .style(*color)
          .data(&[(1.1, 0.0)])
      })
      .collect_vec();

    datasets
  }
}

impl Component for Cpu {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    self.fetch_data()?;
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![
        Constraint::Length(1), // first row
        Constraint::Min(0),
      ])
      .split(area);

    let rect = rects[0];

    let datasets = self.get_cpu_datasets();

    let x_axis = Axis::default().title("X Axis".red()).style(Style::default().white());
    let y_axis = Axis::default().title("Y Axis".red()).style(Style::default().white()).bounds([0.0, self.max_usage]);

    // Create the chart and link all the parts together
    let chart = Chart::new(datasets).block(Block::new().title("Chart")).x_axis(x_axis).y_axis(y_axis);

    f.render_widget(chart, area);
    Ok(())
  }
}
