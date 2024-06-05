use color_eyre::eyre::{ErrReport, Result};
use serde::{Deserialize, Serialize};

use super::{
  cpu::{get_cpu_info, CpuDataCollection},
  disks::{get_disk_info, DiskDataCollection},
  network::{get_network_info, NetworkDataCollection},
  processes::{get_process_info, ProcessDataCollection},
};

// TODO Should the data collection be broken into some combination if Traits?
// Generic Trait for collecting different data
// pub trait DataCollector {
//   type Output;
//   type Params;
//   fn collect(&self, params: Self::Params) -> Self::Output;
// }

/// Represents the source of system information, including system, disk, and network data.
#[derive(Debug)]
pub struct SysinfoSource {
  pub(crate) system: sysinfo::System,
  pub(crate) disks: sysinfo::Disks,
  pub(crate) networks: sysinfo::Networks,
}

impl Default for SysinfoSource {
  /// Creates a new `SysinfoSource` with refreshed lists of disks and networks.
  fn default() -> Self {
    use sysinfo::*;
    Self {
      system: System::new_with_specifics(RefreshKind::new()),
      disks: Disks::new_with_refreshed_list(),
      networks: Networks::new_with_refreshed_list(),
    }
  }
}

/// A structure holding collected data from various system components.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DataCollected {
  pub cpu: Option<CpuDataCollection>,
  pub processes: Option<ProcessDataCollection>,
  pub disk: Option<DiskDataCollection>,
  pub networks: Option<NetworkDataCollection>,
}

/// Manages the collection of data from the system, including CPU, processes, disks, and networks.
#[derive(Debug)]
pub struct DataCollector {
  pub data: DataCollected,
  sys: SysinfoSource,
}

impl Default for DataCollector {
  /// Creates a new `DataCollector` with default values.
  fn default() -> Self {
    Self::new()
  }
}

impl DataCollector {
  /// Creates a new `DataCollector` instance with default data and system information source.
  pub fn new() -> Self {
    DataCollector { data: DataCollected::default(), sys: SysinfoSource::default() }
  }

  /// Updates all the collected data by refreshing system information and then collecting
  /// data for CPU, processes, disks, and networks.
  pub fn update_data(&mut self) {
    self.refresh_sysinfo();

    self.data.cpu = self.update_info(|sys: &SysinfoSource| get_cpu_info(&sys.system), "CPU");
    self.data.processes = self.update_info(|sys: &SysinfoSource| get_process_info(&sys.system), "Process");
    self.data.disk = self.update_info(|sys: &SysinfoSource| get_disk_info(&sys.disks), "Disk");
    self.data.networks = self.update_info(|sys: &SysinfoSource| get_network_info(&sys.networks), "Network");
  }

  /// Refreshes system information, including networks, CPU, processes, and disks.
  fn refresh_sysinfo(&mut self) {
    self.sys.networks.refresh();

    self.sys.system.refresh_cpu();

    self.sys.system.refresh_processes();

    self.sys.disks.refresh_list();
    self.sys.disks.refresh();

    // self.sys.networks.refresh_list();
  }

  /// Collects information using the provided function and logs the result.
  ///
  /// # Arguments
  ///
  /// * `get_info` - A function that collects the information from the `SysinfoSource`.
  /// * `info_type` - A string representing the type of information being collected.
  ///
  /// # Returns
  ///
  /// An `Option` containing the collected data if successful, or `None` if there was an error.

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
