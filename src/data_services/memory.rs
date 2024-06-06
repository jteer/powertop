use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryData {
  // the amount of free RAM in bytes
  pub free_ram: u64,
  pub total_ram: u64,
  // the amount of free SWAP in bytes
  pub free_swap: u64,
  pub total_swap: u64,
}

impl MemoryData {
  pub fn usage_percentages(&self) -> (f64, f64) {
    let used_ram = self.total_ram - self.free_ram;
    let ram_usage = (used_ram as f64 / self.total_ram as f64) * 100.0;

    let used_swap = self.total_swap - self.free_swap;
    let swap_usage = (used_swap as f64 / self.total_swap as f64) * 100.0;

    (ram_usage, swap_usage)
  }
}

pub fn get_memory_info(system: &System) -> Result<MemoryData> {
  Ok(MemoryData {
    // available_ram: system.available_memory(),
    free_ram: system.free_memory(),
    total_ram: system.total_memory(),
    free_swap: system.free_swap(),
    total_swap: system.total_swap(),
  })
}
