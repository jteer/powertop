use std::{any::Any, collections::HashMap, ffi::OsStr, path::Path};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sysinfo::{Disk, Disks, Pid, Process, System};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskData {
  pub name: String,
  pub kind: String,
  pub file_system: String,
  pub total_space: u64,
  pub available_space: u64,
  pub is_removable: bool,
  pub mount_path: String,
}
pub type DiskDataCollection = Vec<DiskData>;

impl From<&Disk> for DiskData {
  fn from(disk: &Disk) -> Self {
    DiskData {
      name: disk.name().to_string_lossy().into_owned(),
      kind: disk.kind().to_string(),
      file_system: disk.file_system().to_string_lossy().into_owned(),

      total_space: disk.total_space(),         // in bytes
      available_space: disk.available_space(), // in bytes
      is_removable: disk.is_removable(),
      mount_path: disk.mount_point().to_string_lossy().into_owned(),
    }
  }
}

// Wrapper so we can create From<>
struct DisksWrapper<'a> {
  disks: &'a Disks,
}
impl<'a> From<DisksWrapper<'a>> for DiskDataCollection {
  fn from(wrapper: DisksWrapper<'a>) -> Self {
    wrapper.disks.iter().map(|disk| disk.into()).collect()
  }
}

pub fn get_disk_info(disks: &Disks) -> Result<DiskDataCollection> {
  Ok(DisksWrapper { disks }.into())
}
