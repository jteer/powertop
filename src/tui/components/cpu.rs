use std::{
  cmp,
  collections::{HashMap, VecDeque},
  default,
  ops::{Add, Sub},
  sync::Arc,
  time::Instant,
};

use clap::builder::styling::AnsiColor;
use color_eyre::{eyre::Result, owo_colors::OwoColorize};
use itertools::Itertools;
use libc::group;
use ratatui::{prelude::*, widgets::*};
use sysinfo::System;

use super::Component;
use crate::{
  data_services::cpu::{get_cpu_info, CpuData, CpuDataCollection},
  tui::action::Action,
};

const MAX_DATA_POINTS: usize = 50;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CpuStats {
  pub max_usage: f64,
  pub min_x: f64,
  pub max_x: f64,
  // Map of Cpu Usage (Cpu_Name, (i, usage))
  pub cpu_groups: HashMap<String, VecDeque<(f64, f64)>>,
  pub points: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CpuGraphType {
  #[default]
  LineChart,
  BarChart,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cpu {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: CpuDataCollection,
  cpu_stats: CpuStats,
  graph_type: CpuGraphType,
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
      cpu_stats: CpuStats { max_usage: 0.0, cpu_groups: HashMap::new(), min_x: 0.0, max_x: 0.0, points: 0 },
      graph_type: CpuGraphType::BarChart,
    }
  }

  fn update_data_stats(&mut self, new_data: Vec<CpuData>) {
    log::debug!("Updating CPU Component with new data of len: {:?}", new_data.len());

    let mut max_x = f64::NEG_INFINITY;
    let mut max_from_new_data: f64 = 0.0;

    // if self.cpu_stats.points + 1 >= MAX_DATA_POINTS {
    //   self.cpu_stats = CpuStats { max_x: self.cpu_stats.max_x, max_y: self.cpu_stats.max_y, ..Default::default() }
    // }

    self.cpu_stats.points += 1;

    max_x = max_x.max(self.cpu_stats.points as f64);

    // Should contain one new item for each cpu
    for (i, data) in new_data.iter().enumerate() {
      max_from_new_data = max_from_new_data.max(data.cpu_usage);

      if self.cpu_stats.cpu_groups.contains_key(&data.cpu_name) {
        if let Some(existing_entry) = self.cpu_stats.cpu_groups.get_mut(&data.cpu_name) {
          // TODO Currently this does not clear from the deque
          let x_pos = self.cpu_stats.points;
          existing_entry.push_back((x_pos as f64, data.cpu_usage));
        }
      } else {
        let mut deque = VecDeque::with_capacity(MAX_DATA_POINTS);
        deque.push_back((0.0_f64, data.cpu_usage));
        self.cpu_stats.cpu_groups.insert(data.cpu_name.clone(), deque);
      }
    }

    self.cpu_stats.max_usage = max_from_new_data.max(self.cpu_stats.max_usage);
    self.cpu_stats.max_x = max_x.max(self.cpu_stats.max_x);

    // self.collected_data.append(&mut new_data);
  }

  fn get_bar_chart_datasets(&mut self) -> Vec<Bar> {
    self
      .cpu_stats
      .cpu_groups
      .iter()
      .sorted_by_key(|x| x.0)
      .map(|x| -> Bar {
        match x.1.back() {
          Some(d) => Bar::default().label(format!("CPU{:<4}", x.0.to_string()).into()).value(d.1 as u64),
          None => todo!("handle failed to map cpu value to bar value"),
        }
      })
      .collect_vec()
  }

  fn get_line_chart_datasets(&mut self) -> Vec<Dataset> {
    // TODO: Add more colors so that each cpu consistently keeps the same color
    let colors = [
      Style::default().cyan(),
      Style::default().magenta(),
      Style::default().yellow(),
      Style::default().green(),
      Style::default().blue(),
      Style::default().red(),
      Style::default().black(),
      Style::default().gray(),
      Style::default().dark_gray(),
    ];

    let mut color_iter = colors.iter().cycle();

    let mut datasets = self
      .cpu_stats
      .cpu_groups
      .iter()
      .sorted_by_key(|x| x.0)
      .map(|x| {
        let color = color_iter.next().unwrap();
        Dataset::default()
          .name(x.0.to_string())
          .marker(symbols::Marker::Dot)
          .graph_type(GraphType::Line)
          .style(*color)
          .data(x.1.as_slices().0)
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
      match data.cpu {
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
        Constraint::Percentage(50), // Top row spans whole width
        Constraint::Percentage(50), // Bottom row split 50/50
      ])
      .split(area);
    let top_row = rects[0];
    let top_row_rects = Layout::default()
      .direction(Direction::Horizontal)
      .constraints(vec![
        Constraint::Percentage(50), // Top row spans whole width
        Constraint::Percentage(50), // Bottom row split 50/50
      ])
      .split(top_row);
    let cpu_rect = top_row_rects[0];
    // TODO: CPU Ordering on both graphs
    // TODO: Handle Data Cleaning
    // TODO: Each of these charts could be moved into its own "Widget" module as an abstraction over ratatui so it can be easy to implement new charts
    // TODO: The data for each of these could be abstracted into some
    match self.graph_type {
      // TODO: Handle correctly updating chart when data points exceed MAX_DATA_POINTS
      CpuGraphType::LineChart => {
        let x_lower_bound =
          if self.cpu_stats.points >= MAX_DATA_POINTS { self.cpu_stats.points - MAX_DATA_POINTS } else { 0 };
        let x_axis = Axis::default()
          .style(Style::default().white())
          .bounds([x_lower_bound as f64, self.cpu_stats.points as f64])
          .labels(vec!["0.0".into(), MAX_DATA_POINTS.to_string().into()]);

        // usage
        let y_axis = Axis::default()
          .style(Style::default().white())
          .bounds([0.0, 100.0])
          .labels(vec!["0.0".into(), "100.0".into()]);

        let datasets = self.get_line_chart_datasets();

        let chart = Chart::new(datasets)
          .block(Block::bordered().title("CPU"))
          .x_axis(x_axis)
          .y_axis(y_axis)
          .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)))
          .legend_position(Some(LegendPosition::TopRight));

        frame.render_widget(chart, cpu_rect);
      },
      CpuGraphType::BarChart => {
        // TODO: For each bar draw the previous value + new value to show change?
        let dataset = self.get_bar_chart_datasets();
        let chart = BarChart::default()
          .block(Block::bordered().title("CPU"))
          .bar_width(4)
          .bar_gap(3)
          .group_gap(3)
          .bar_style(Style::new().blue().on_black())
          .value_style(Style::new().white().bold())
          .label_style(Style::new().black())
          // TODO impl Into
          // .data(self.cpu_stats.cpu_groups)
          .data(BarGroup::default().bars(&dataset))
          .max(100);

        frame.render_widget(chart, cpu_rect);
      },
    };

    Ok(())
  }
}
