use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use super::cpu::{get_cpu_info, CpuDataCollection};

// Generic Trait for collecting different data
// pub trait DataCollector {
//   type Output;
//   type Params;
//   fn collect(&self, params: Self::Params) -> Self::Output;
// }

#[derive(Debug)]
pub struct SysinfoSource {
  pub(crate) system: sysinfo::System,
}

impl Default for SysinfoSource {
  fn default() -> Self {
    use sysinfo::*;
    Self { system: System::new_with_specifics(RefreshKind::new()) }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DataCollected {
  pub cpu: Option<CpuDataCollection>,
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
    self.update_sysinfo();

    self.update_cpu();
  }

  fn update_sysinfo(&mut self) {
    // TODO Make configurable / Refresh only every 60 sec
    // const REFRESH_TIME = Duration::from_secs(60);
    // let refresh_start = Instant::now();

    self.sys.system.refresh_cpu();
  }

  fn update_cpu(&mut self) {
    let cpu = get_cpu_info(&self.sys.system);
    match cpu {
      Ok(d) => self.data.cpu = Some(d),
      Err(_) => todo!(),
    }
  }
}