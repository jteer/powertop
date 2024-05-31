use std::collections::HashMap;

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, Process, System};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessData {
  pub pid: u32,
  pub parent: Option<u32>,
  pub name: String,
  pub status: String,
  pub cpu_usage: f32,
}

pub type ProcessDataCollection = Vec<ProcessData>;

pub trait IntoProcessDataCollection {
  fn into_process_data_collection(self) -> ProcessDataCollection;
}

impl IntoProcessDataCollection for &HashMap<Pid, Process> {
  fn into_process_data_collection(self) -> ProcessDataCollection {
    self
      .iter()
      .map(|(pid, process)| {
        ProcessData {
          pid: pid.as_u32(),
          name: process.name().to_string(),
          parent: process.parent().map(Pid::as_u32),
          status: process.status().to_string(),
          cpu_usage: process.cpu_usage(),
        }
      })
      .collect()
  }
}

pub fn get_process_info(system: &System) -> Result<ProcessDataCollection> {
  Ok(system.processes().into_process_data_collection())
}
