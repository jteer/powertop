use std::time::Instant;

use color_eyre::{
  eyre::{Ok, Result},
  owo_colors::OwoColorize,
};
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{
  data_services::processes::{ProcessData, ProcessDataCollection},
  tui::{action::Action, ui::Frame},
};

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessTable {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: ProcessDataCollection,
}

impl From<ProcessData> for Row<'static> {
  fn from(val: ProcessData) -> Self {
    Row::new(vec![
      val.pid.to_string(),
      match val.parent {
        Some(p) => p.to_string(),
        None => "-".to_string(),
      },
      val.name,
      val.status,
      format!("{:.3}", val.cpu_usage),
    ])
  }
}

impl ProcessData {
  // TODO: Better way to create headers from struct
  fn headers() -> Vec<&'static str> {
    vec!["PID", "Parent", "Name", "Status", "CPU Usage"]
  }

  fn column_widths() -> Vec<Constraint> {
    // TODO: Should we use Constraint::Min() ?
    vec![
      Constraint::Length(4),  // PID column minimum width
      Constraint::Length(6),  // Parent column minimum width
      Constraint::Length(12), // Name column minimum width
      Constraint::Length(12), // Status column minimum width
      Constraint::Length(10), // CPU Usage column minimum width
    ]
  }
}

impl Default for ProcessTable {
  fn default() -> Self {
    Self::new()
  }
}

impl ProcessTable {
  pub fn new() -> Self {
    Self { app_start_time: Instant::now(), render_start_time: Instant::now(), collected_data: [].to_vec() }
  }

  fn update_data_stats(&mut self, new_data: ProcessDataCollection) {
    self.collected_data = new_data;
  }
}

impl Component for ProcessTable {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::DataUpdate(data) = action {
      match data.processes {
        Some(d) => self.update_data_stats(d),
        None => {
          log::debug!("Received Action with no data.")
        },
      }
    }
    Ok(None)
  }

  fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) -> Result<()> {
    // TODO: DRY for Layout
    // Option 1: Each component creates Layout the same way and each uses a different rect (rects[0], ...)
    // Option 2: App creates Layout and passes rects down to components, each component knows which rect to use
    // Option 3: App creates Layout and passes a single rect down that the component uses
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
        Constraint::Percentage(50), // Left half
        Constraint::Percentage(50), // Right half
      ])
      .split(rects[1]);

    // TODO: Do we need to clone?
    let rows: Vec<Row> = self.collected_data.clone().into_iter().map(Into::into).collect();
    let col_widths = ProcessData::column_widths();
    let header = Row::new(ProcessData::headers()).style(Style::default().bold().underlined()).bottom_margin(1);

    let table = Table::new(rows, col_widths)
      .block(Block::bordered().title("Processes"))
      .column_spacing(3)
      .style(Style::default().white())
      .header(header)
      .highlight_style(Style::default().reversed())
      .highlight_symbol(">>");

    frame.render_widget(table, bottom_row_rects[0]);

    Ok(())
  }
}
