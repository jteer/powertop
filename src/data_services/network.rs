use std::{any::Any, collections::HashMap, ffi::OsStr, path::Path};

use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use sysinfo::Networks;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkData {
  pub interface_name: String,
  pub mac_address: String,
  // Returns the number of received bytes since the last refresh.
  pub received: u64,
  pub total_received: u64,
  pub total_packets_received: u64,

  // Returns the number of transmitted bytes since the last refresh.
  pub transmitted: u64,
  pub total_transmitted: u64,
  pub total_packets_transmitted: u64,
}

pub type NetworkDataCollection = Vec<NetworkData>;

impl From<(&String, &sysinfo::NetworkData)> for NetworkData {
  fn from((interface_name, net_data): (&String, &sysinfo::NetworkData)) -> Self {
    NetworkData {
      interface_name: interface_name.to_owned(),
      mac_address: net_data.mac_address().to_string(),

      received: net_data.received(),
      total_received: net_data.total_received(),
      total_packets_received: net_data.total_packets_received(),

      // Returns the number of transmitted bytes since the last refresh.
      transmitted: net_data.transmitted(),
      total_transmitted: net_data.total_transmitted(),
      total_packets_transmitted: net_data.total_packets_transmitted(),
    }
  }
}

struct NetworkDataWrapper<'a> {
  networks: &'a Networks,
}

impl<'a> From<NetworkDataWrapper<'a>> for NetworkDataCollection {
  fn from(wrapper: NetworkDataWrapper<'a>) -> Self {
    wrapper.networks.iter().map(|network: (&String, &sysinfo::NetworkData)| network.into()).collect()
  }
}

pub fn get_network_info(networks: &Networks) -> Result<NetworkDataCollection> {
  Ok(NetworkDataWrapper { networks }.into())
}
