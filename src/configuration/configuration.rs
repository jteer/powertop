use std::{collections::HashMap, fmt, path::PathBuf};

use color_eyre::eyre::Result;
use config::Value;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use ratatui::style::{Color, Modifier, Style};
use serde::{
  de::{self, Deserializer, MapAccess, Visitor},
  Deserialize, Serialize,
};
use serde_json::Value as JsonValue;
use tracing::error;
use tracing_error::ErrorLayer;
use tracing_subscriber::{self, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer};

use super::{keybindings::KeyBindings, styles::Styles};

const CONFIG: &str = include_str!("../../.config/config.json5");

pub const VERSION_MESSAGE: &str =
  concat!(env!("CARGO_PKG_VERSION"), "-", env!("VERGEN_GIT_DESCRIBE"), " (", env!("VERGEN_BUILD_DATE"), ")");

lazy_static! {
  pub static ref PROJECT_NAME: String = env!("CARGO_CRATE_NAME").to_uppercase().to_string();
  pub static ref DATA_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_DATA", PROJECT_NAME.clone())).ok().map(PathBuf::from);
  pub static ref CONFIG_FOLDER: Option<PathBuf> =
    std::env::var(format!("{}_CONFIG", PROJECT_NAME.clone())).ok().map(PathBuf::from);
}

fn project_directory() -> Option<ProjectDirs> {
  ProjectDirs::from("com", "jteer", env!("CARGO_PKG_NAME"))
}

pub fn get_data_dir() -> PathBuf {
  let directory = if let Some(s) = DATA_FOLDER.clone() {
    s
  } else if let Some(proj_dirs) = project_directory() {
    proj_dirs.data_local_dir().to_path_buf()
  } else {
    PathBuf::from(".").join(".data")
  };
  directory
}

pub fn get_config_dir() -> PathBuf {
  let directory = if let Some(s) = CONFIG_FOLDER.clone() {
    s
  } else if let Some(proj_dirs) = project_directory() {
    proj_dirs.config_local_dir().to_path_buf()
  } else {
    PathBuf::from(".").join(".config")
  };
  directory
}

pub fn version() -> String {
  let author = clap::crate_authors!();

  // let current_exe_path = PathBuf::from(clap::crate_name!()).display().to_string();
  let config_dir_path = get_config_dir().display().to_string();
  let data_dir_path = get_data_dir().display().to_string();

  format!(
    "\
  {VERSION_MESSAGE}
  
  Authors: {author}
  
  Config directory: {config_dir_path}
  Data directory: {data_dir_path}"
  )
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
  #[serde(default)]
  pub _data_dir: PathBuf,
  #[serde(default)]
  pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default, flatten)]
  pub config: AppConfig,
  #[serde(default)]
  pub keybindings: KeyBindings,
  #[serde(default)]
  pub styles: Styles,
}

impl Config {
  pub fn new() -> Result<Self, config::ConfigError> {
    let default_config: Config = json5::from_str(CONFIG).unwrap();
    let data_dir = get_data_dir();
    let config_dir = get_config_dir();
    let mut builder = config::Config::builder()
      .set_default("_data_dir", data_dir.to_str().unwrap())?
      .set_default("_config_dir", config_dir.to_str().unwrap())?;

    let config_files = [
      ("config.json5", config::FileFormat::Json5),
      ("config.json", config::FileFormat::Json),
      ("config.yaml", config::FileFormat::Yaml),
      ("config.toml", config::FileFormat::Toml),
      ("config.ini", config::FileFormat::Ini),
    ];
    let mut found_config = false;
    for (file, format) in &config_files {
      builder = builder.add_source(config::File::from(config_dir.join(file)).format(*format).required(false));
      if config_dir.join(file).exists() {
        found_config = true
      }
    }
    if !found_config {
      log::error!("No configuration file found. Application may not behave as expected");
    }

    let mut cfg: Self = builder.build()?.try_deserialize()?;

    for (mode, default_bindings) in default_config.keybindings.iter() {
      let user_bindings = cfg.keybindings.entry(*mode).or_default();
      for (key, cmd) in default_bindings.iter() {
        user_bindings.entry(key.clone()).or_insert_with(|| cmd.clone());
      }
    }
    for (mode, default_styles) in default_config.styles.iter() {
      let user_styles = cfg.styles.entry(*mode).or_default();
      for (style_key, style) in default_styles.iter() {
        user_styles.entry(style_key.clone()).or_insert_with(|| *style);
      }
    }

    Ok(cfg)
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;

  use super::*;
  use crate::{configuration::keybindings::parse_key_sequence, tui::{action::Action, mode::Mode}};

  #[test]
  fn test_config() -> Result<()> {
    let c = Config::new()?;
    assert_eq!(
      c.keybindings.get(&Mode::Home).unwrap().get(&parse_key_sequence("<q>").unwrap_or_default()).unwrap(),
      &Action::Quit
    );
    Ok(())
  }
}
