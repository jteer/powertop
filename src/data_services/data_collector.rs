use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use super::{
  cpu::{get_cpu_info, CpuDataCollection},
  disks::{get_disk_info, DiskDataCollection},
  processes::{get_process_info, ProcessDataCollection},
};

// Generic Trait for collecting different data
// pub trait DataCollector {
//   type Output;
//   type Params;
//   fn collect(&self, params: Self::Params) -> Self::Output;
// }

#[derive(Debug)]
pub struct SysinfoSource {
  pub(crate) system: sysinfo::System,
  pub(crate) disks: sysinfo::Disks,
}

impl Default for SysinfoSource {
  fn default() -> Self {
    use sysinfo::*;
    Self { system: System::new_with_specifics(RefreshKind::new()), disks: Disks::new_with_refreshed_list() }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DataCollected {
  pub cpu: Option<CpuDataCollection>,
  pub processes: Option<ProcessDataCollection>,
  pub disk: Option<DiskDataCollection>,
}

#[derive(Debug)]
pub struct DataCollector {
  pub data: DataCollected,
  sys: SysinfoSource,
}

impl Default for DataCollector {
  fn default() -> Self {
    Self::new()
  }
}

impl DataCollector {
  pub fn new() -> Self {
    DataCollector { data: DataCollected::default(), sys: SysinfoSource::default() }
  }

  pub fn update_data(&mut self) {
    self.refresh_sysinfo();

    self.update_cpu();
    self.update_process_info();
    self.update_disk_info();
  }

  fn refresh_sysinfo(&mut self) {
    // TODO Make configurable / Refresh only every 60 sec
    // const REFRESH_TIME = Duration::from_secs(60);
    // let refresh_start = Instant::now();

    self.sys.system.refresh_cpu();

    self.sys.system.refresh_processes();

    self.sys.disks.refresh_list();
    self.sys.disks.refresh();
  }

  fn update_cpu(&mut self) {
    let cpu = get_cpu_info(&self.sys.system);
    match cpu {
      Ok(d) => self.data.cpu = Some(d),
      Err(_) => todo!(),
    }
  }

  fn update_process_info(&mut self) {
    let process_data = get_process_info(&self.sys.system);
    match process_data {
      Ok(d) => self.data.processes = Some(d),
      Err(_) => todo!(),
    }
  }

  fn update_disk_info(&mut self) {
    let disk_info = get_disk_info(&self.sys.disks);
    match disk_info {
      Ok(d) => self.data.disk = Some(d),
      Err(_) => todo!(),
    }
  }
}
