#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod app;
pub mod cli;
pub mod configuration;
pub mod data_services;
pub mod logging;
pub mod tui;
pub mod utils;
use std::sync::{Arc, Mutex};

use clap::Parser;
use cli::Cli;
use color_eyre::eyre::Result;
use logging::logging::initialize_logging;
use sysinfo::System;

use crate::{app::App, utils::initialize_panic_handler};

async fn tokio_main() -> Result<()> {
  initialize_logging()?;

  initialize_panic_handler()?;
  let args = Cli::parse();

  let mut app = App::new(args.tick_rate, args.frame_rate)?;
  app.run().await?;

  Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
  if let Err(e) = tokio_main().await {
    eprintln!("{} error: Something went wrong", env!("CARGO_PKG_NAME"));
    Err(e)
  } else {
    Ok(())
  }
}
