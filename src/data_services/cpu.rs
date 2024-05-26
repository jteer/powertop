use sysinfo::System;
use color_eyre::eyre::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct CpuData {
  pub cpu_name: String,
  pub vendor_id: String,
  pub brand: String,
  pub cpu_usage: f64,
}
pub type CpuDataCollection = Vec<CpuData>;

// https://crates.io/crates/sysinfo
// Supports
// Android
// FreeBSD
// iOS
// Linux
// macOS
// Raspberry Pi
// Windows
pub fn get_cpu_info(system: &System) -> Result<CpuDataCollection> {
    // TODO Move refresh higher in workflow
    // system.refresh_cpu();

    let cpu_info: Vec<CpuData> = system
        .cpus()
        .iter()
        .enumerate()
        .map(|(_i, cpu)| CpuData {
            cpu_usage: cpu.cpu_usage() as f64,
            cpu_name: cpu.name().to_owned(),
            vendor_id: cpu.vendor_id().to_owned(),
            brand: cpu.brand().to_owned(),
        })
        .collect();

    Ok(Vec::from(cpu_info))
}
