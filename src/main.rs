#![doc = include_str!("../README.md")]

use std::{borrow::Cow, env};

use async_std::{channel, future::pending, task};
use env_logger::Env;
use log::{error, info};
use pico_args::Arguments;

mod fs;
mod services;
mod tcp;
mod udp;
mod utils;

fn main() {
	let args = {
		let mut args = Arguments::from_env();

		let log = match args.opt_value_from_str("--log") {
			Ok(Some(log)) => (Cow::Owned(log), None, true),
			Ok(None) => (Cow::Borrowed("error"), None, false),
			Err(e) => (
				Cow::Borrowed("error"),
				Some(format!(
					"Couldn't parse contents of the `--log` command line option: {e}"
				)),
				false,
			),
		};

		let log_style = match args.opt_value_from_str("--log-style") {
			Ok(Some(log)) => (Cow::Owned(log), None),
			Ok(None) => (Cow::Borrowed("auto"), None),
			Err(e) => (
				Cow::Borrowed("auto"),
				Some(format!(
					"Couldn't parse contents of the `--log-style` command line option: {e}"
				)),
			),
		};

		let env = Env::new()
			.filter_or("SIMPLE_PROTOCOLS_LOG", log.0)
			.write_style_or("SIMPLE_PROTOCOLS_LOG_STYLE", log_style.0);

		env_logger::init_from_env(env);

		if let Some(msg) = log.1 {
			error!("{msg}");
		}

		if let Some(msg) = log_style.1 {
			error!("{msg}");
		}

		if !log.2 && env::var_os("SIMPLE_PROTOCOLS_LOG").is_none() {
			eprintln!("Logging is not configured, and only errors will be logged by default");
			eprintln!(
				"Configure logging using the `SIMPLE_PROTOCOLS_LOG` environment variable or the \
				 `--log` command line option"
			);
		}

		args
	};

	let (shutdown_tx, shutdown_rx) = channel::bounded(1);
	if let Err(e) = ctrlc::set_handler(move || {
		if let Err(e) = shutdown_tx.send_blocking(()) {
			error!("Couldn't handle CTRL-C, the server may not gracefully exit on CTRL-C: {e}");
		};
	}) {
		error!("Couldn't set CTRL-C handler, the server may not gracefully exit on CTRL-C: {e}");
	};

	task::block_on(async {
		services::spawn_all(args);

		info!("Simple Protocols Started");

		let Ok(()) = shutdown_rx.recv().await else {
			error!("Couldn't use CTRL-C handler, the server may not gracefully exit on CTRL-C");
			pending::<()>().await;
			unreachable!()
		};
	});

	info!("Simple Protocols Exiting");
}
