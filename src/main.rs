#![doc = include_str!("../README.md")]

use std::env;

use async_std::future::pending;
use log::info;

mod fs;
mod services;
mod tcp;
mod udp;
mod utils;

#[async_std::main]
async fn main() {
	env_logger::init_from_env("SIMPLE_PROTOCOLS_LOG");

	if env::var_os("SIMPLE_PROTOCOLS_LOG").is_none() {
		eprintln!("Logging is not configured, and only errors will be logged by default");
		eprintln!("Configure logging using the `SIMPLE_PROTOCOLS_LOG` environment variable");
	}

	services::spawn_all();
	info!("Simple Protocols Started");
	pending::<()>().await
}
