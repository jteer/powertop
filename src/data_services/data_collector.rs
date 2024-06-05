use std::time::Duration;

use color_eyre::eyre::{ErrReport, Result};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use super::{
  cpu::{get_cpu_info, CpuDataCollection},
  disks::{get_disk_info, DiskDataCollection},
  network::{get_network_info, NetworkDataCollection},
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
  pub(crate) networks: sysinfo::Networks,
}

impl Default for SysinfoSource {
  fn default() -> Self {
    use sysinfo::*;
    Self {
      system: System::new_with_specifics(RefreshKind::new()),
      disks: Disks::new_with_refreshed_list(),
      networks: Networks::new_with_refreshed_list(),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DataCollected {
  pub cpu: Option<CpuDataCollection>,
  pub processes: Option<ProcessDataCollection>,
  pub disk: Option<DiskDataCollection>,
  pub networks: Option<NetworkDataCollection>,
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

    // TODO Should this be broken into some combination if Traits?
    self.data.cpu = self.update_info(|sys: &SysinfoSource| get_cpu_info(&sys.system), "CPU");
    self.data.processes = self.update_info(|sys: &SysinfoSource| get_process_info(&sys.system), "Process");
    self.data.disk = self.update_info(|sys: &SysinfoSource| get_disk_info(&sys.disks), "Disk");
    self.data.networks = self.update_info(|sys: &SysinfoSource| get_network_info(&sys.networks), "Network");
  }

  fn refresh_sysinfo(&mut self) {
    self.sys.networks.refresh();

    self.sys.system.refresh_cpu();

    self.sys.system.refresh_processes();

    self.sys.disks.refresh_list();
    self.sys.disks.refresh();

    // self.sys.networks.refresh_list();
  }

  fn update_info<F, T>(&self, get_info: F, info_type: &str) -> Option<T>
  where
    F: Fn(&SysinfoSource) -> Result<T, ErrReport>,
    T: std::fmt::Debug,
  {
    match get_info(&self.sys) {
      Ok(info) => {
        log::debug!("Collected {} Data: {:?}", info_type, info);
        Some(info)
      },
      Err(_) => {
        log::warn!("Failed to collect {} Data", info_type);
        None
      },
    }
  }
}
