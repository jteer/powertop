use std::time::Instant;

use color_eyre::{
  eyre::{Ok, Result},
  owo_colors::OwoColorize,
};
use ratatui::{prelude::*, widgets::*};

use super::Component;
use crate::{
  data_services::disks::{DiskData, DiskDataCollection},
  tui::{action::Action, ui::Frame},
};

#[derive(Debug, Clone, PartialEq)]
pub struct DiskTable {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: DiskDataCollection,
}

impl From<DiskData> for Row<'static> {
  fn from(val: DiskData) -> Self {
    Row::new(vec![
      val.name,
      val.kind,
      val.file_system,
      val.total_space.to_string(),
      val.available_space.to_string(),
      val.is_removable.to_string(),
      val.mount_path,
    ])
  }
}

impl DiskData {
  // TODO: Better way to create headers from struct
  fn headers() -> Vec<&'static str> {
    vec!["Name", "Kind", "File System", "Total (bytes)", "Available (bytes)", "IsRemovable", "Mount"]
  }

  fn column_widths() -> Vec<Constraint> {
    vec![
      Constraint::Length(12),
      Constraint::Length(6),
      Constraint::Length(12),
      Constraint::Length(12),
      Constraint::Length(12),
      Constraint::Length(6),
      Constraint::Length(12),
    ]
  }
}

impl Default for DiskTable {
  fn default() -> Self {
    Self::new()
  }
}

impl DiskTable {
  pub fn new() -> Self {
    Self { app_start_time: Instant::now(), render_start_time: Instant::now(), collected_data: [].to_vec() }
  }

  fn update_data_stats(&mut self, new_data: DiskDataCollection) {
    self.collected_data = new_data;
  }
}

impl Component for DiskTable {
  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    if let Action::DataUpdate(data) = action {
      match data.disk {
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
        Constraint::Percentage(50), // Left half
        Constraint::Percentage(50), // Right half
      ])
      .split(rects[1]);

    let rows: Vec<Row> = self.collected_data.clone().into_iter().map(Into::into).collect();
    let col_widths = DiskData::column_widths();
    let header = Row::new(DiskData::headers()).style(Style::default().bold().underlined()).bottom_margin(1);

    let table = Table::new(rows, col_widths)
      .block(Block::bordered().title("Disk"))
      .column_spacing(3)
      .style(Style::default().white())
      .header(header)
      .highlight_style(Style::default().reversed())
      .highlight_symbol(">>");

    frame.render_widget(table, bottom_row_rects[1]);

    Ok(())
  }
}
