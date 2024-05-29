use std::{sync::Arc, time::Instant};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpuData {
  pub cpu_name: String,
  pub vendor_id: String,
  pub brand: String,
  pub cpu_usage: f64,
}
pub type CpuDataCollection = Vec<CpuData>;

#[derive(Debug, Clone, PartialEq)]
pub struct Cpu {
  app_start_time: Instant,
  render_start_time: Instant,
  collected_data: CpuDataCollection,
}

pub fn get_cpu_info(system: &System) -> Result<CpuDataCollection> {
  // TODO Move refresh higher in workflow
  // system.refresh_cpu();

  let cpu_info: Vec<CpuData> = system
    .cpus()
    .iter()
    .map(|cpu| {
      CpuData {
        cpu_usage: cpu.cpu_usage() as f64,
        cpu_name: cpu.name().to_owned(),
        vendor_id: cpu.vendor_id().to_owned(),
        brand: cpu.brand().to_owned(),
      }
    })
    .collect();

  Ok(cpu_info)
}
