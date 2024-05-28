use std::{cmp, collections::HashMap, ops::Add, sync::Arc, time::Instant};

use color_eyre::eyre::Result;
use itertools::Itertools;
use libc::group;
use ratatui::{prelude::*, widgets::*};
use sysinfo::System;

use super::Component;
use crate::{
  data_services::cpu::{get_cpu_info, CpuData, CpuDataCollection},
  tui::action::Action,
};

const MAX_DATA_POINTS: usize = 20;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CpuStats {
  pub max_usage: f64,
  pub min_x: f64,
  pub max_x: f64,
  pub min_y: f64,
  pub max_y: f64,
  // Map of Cpu Usage (Cpu_Name, (i, usage))
  pub cpu_groups: HashMap<String, Vec<(f64, f64)>>,
  pub points: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cpu {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: CpuDataCollection,
  cpu_stats: CpuStats,
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
      cpu_stats: CpuStats {
        max_usage: 0.0,
        cpu_groups: HashMap::new(),
        min_x: 0.0,
        max_x: 0.0,
        min_y: 0.0,
        max_y: 0.0,
        points: 0,
      },
    }
  }

  fn update_data_stats(&mut self, new_data: Vec<CpuData>) {
    log::info!("Updating with new data of len: {:?}", new_data.len());

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut max_from_new_data: f64 = 0.0;

    // TODO: This clears the whole graph, we should instead use something like VecDeque so we can pop values from the left
    // Which should look like the graph is moving?
    if self.cpu_stats.points + 1 >= MAX_DATA_POINTS {
      self.cpu_stats = CpuStats { max_x: self.cpu_stats.max_x, max_y: self.cpu_stats.max_y, ..Default::default() }
    }

    self.cpu_stats.points += 1;

    for (i, data) in new_data.iter().enumerate() {
      max_from_new_data = max_from_new_data.max(data.cpu_usage);

      min_x = min_x.min((i + 1) as f64);
      max_x = max_x.max((i + 1) as f64);

      min_y = min_y.min(data.cpu_usage);
      max_y = max_y.max(data.cpu_usage);

      // TODO Convert to use Time as x
      // let time_from_start: f64 = (self.render_start_time.duration_since(self.app_start_time).as_millis() as f64).floor();

      if self.cpu_stats.cpu_groups.contains_key(&data.cpu_name) {
        if let Some(existing_entry) = self.cpu_stats.cpu_groups.get_mut(&data.cpu_name) {
          existing_entry.append(&mut vec![((existing_entry.len() + 1) as f64, data.cpu_usage)]);
        }
      } else {
        self.cpu_stats.cpu_groups.insert(data.cpu_name.clone(), vec![(1.0_f64, data.cpu_usage)]);
      }
    }

    self.cpu_stats.max_usage = max_from_new_data.max(self.cpu_stats.max_usage);
    self.cpu_stats.min_x = min_x.min(self.cpu_stats.min_x);
    self.cpu_stats.max_x = max_x.max(self.cpu_stats.max_x);
    self.cpu_stats.min_y = min_y.min(self.cpu_stats.min_y);
    self.cpu_stats.max_y = max_y.max(self.cpu_stats.max_y);

    // self.collected_data.append(&mut new_data);
  }

  fn get_datasets(&mut self) -> Vec<Dataset> {
    let colors = [
      Style::default().cyan(),
      Style::default().magenta(),
      Style::default().yellow(),
      Style::default().green(),
      Style::default().blue(),
      Style::default().red(),
    ];

    let mut color_iter = colors.iter().cycle();

    let mut datasets = self
      .cpu_stats
      .cpu_groups
      .iter()
      .map(|x| {
        let color = color_iter.next().unwrap();
        Dataset::default()
          .name(x.0.to_string())
          .marker(symbols::Marker::Dot)
          .graph_type(GraphType::Line)
          .style(*color)
          .data(x.1)
      })
      .collect_vec();

    if cfg!(debug_assertions) {
      datasets.push(
        Dataset::default()
          .name("test_data")
          .marker(symbols::Marker::Dot)
          .graph_type(GraphType::Line)
          .style(Style::default().red())
          .data(&[(4.0, 5.0), (5.0, 8.0), (7.66, 13.5)]),
      );
    }

    datasets
  }
}

impl Component for Cpu {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::DataUpdate(data) = action {
      let data = *data;
      let cpu_data = data.cpu;

      match cpu_data {
        Some(d) => {
          log::info!("Received Action event with data: {:?}", d.len());
          self.update_data_stats(d)
        },
        None => todo!(),
      }
    }
    Ok(None)
  }

  fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
    let rects = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![
        Constraint::Length(1), // first row
        Constraint::Min(0),
      ])
      .split(area);

    let rect = rects[0];

    let x_axis = Axis::default()
      .style(Style::default().white())
      .bounds([0.0, MAX_DATA_POINTS as f64])
      .labels(vec!["0.0".into(), MAX_DATA_POINTS.to_string().into()]);

    // Create the Y axis and define its properties
    let y_axis = Axis::default()
      .style(Style::default().white())
      .bounds([self.cpu_stats.min_y, self.cpu_stats.max_y])
      .labels(vec!["0.0".into(), self.cpu_stats.max_y.round().to_string().into()]);

    let datasets = self.get_datasets();

    // Create the chart and link all the parts together
    let chart = Chart::new(datasets)
      .block(Block::new().title("CPU").border_style(Style::new().blue()))
      .x_axis(x_axis)
      .y_axis(y_axis)
      .legend_position(Some(LegendPosition::TopRight));

    frame.render_widget(chart, area);
    Ok(())
  }
}
