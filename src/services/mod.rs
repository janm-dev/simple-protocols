//! Service protocol implementations

pub use std::future::Future;
use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	pin::Pin,
	task::{Context, Poll},
};

use log::info;
use pico_args::Arguments;

// Declare the modules here because rust-analyzer wasn't too happy with
// declaring them inside of the `service` macro
#[cfg(feature = "active")]
mod active;
#[cfg(feature = "chargen")]
mod chargen;
#[cfg(feature = "daytime")]
mod daytime;
#[cfg(feature = "discard")]
mod discard;
#[cfg(feature = "echo")]
mod echo;
#[cfg(feature = "gopher")]
mod gopher;
#[cfg(any(feature = "message-1", feature = "message-2"))]
mod message;
#[cfg(feature = "qotd")]
mod qotd;
#[cfg(feature = "time")]
mod time;

#[derive(Debug, Clone, Copy)]
pub enum ServiceRet {}

#[derive(Debug, Clone, Copy)]
pub enum NoFuture {}

impl Future for NoFuture {
	type Output = ServiceRet;

	fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
		Poll::Pending
	}
}

#[derive(Debug)]
pub struct Config {
	pub base_port: u16,
	pub hostname: Option<String>,
}

impl Config {
	pub fn from_args(mut args: Arguments) -> Result<&'static Self, anyhow::Error> {
		let cfg = Self {
			base_port: args.opt_value_from_str("--base-port")?.unwrap_or(0),
			hostname: args.opt_value_from_str("--hostname")?,
		};

		Ok(Box::leak(Box::new(cfg)))
	}
}

#[derive(Debug)]
pub enum ServiceErr {
	/// The configuration option `config_name` is needed but was not specified,
	/// the service `service_name` can't start
	MissingConfig {
		service_name: &'static str,
		config_name: &'static str,
	},
	/// The service is not available over this protocol and no handler task
	/// needs to be spawned
	NoHandler,
	/// The service usually runs on `usual_port`, and an overflow was
	/// encountered while adding the `base_port`, and the service can't start
	PortTooHigh {
		service_name: &'static str,
		usual_port: u16,
		base_port: u16,
	},
	/// Service initialization encountered another error
	Other(anyhow::Error),
}

impl<E: Into<anyhow::Error>> From<E> for ServiceErr {
	fn from(value: E) -> Self {
		Self::Other(value.into())
	}
}

impl Display for ServiceErr {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::MissingConfig {
				service_name,
				config_name,
			} => f.write_fmt(format_args!(
				"the {service_name} service requires the \"{config_name}\" configuration option, \
				 but it was not supplied and the service could not start (rerun with \
				 \"--{config_name} [VALUE]\" to enable this service)"
			)),
			Self::NoHandler => f.write_str(
				"the service has no handler over this protocol (this error is expected in some \
				 cases and should never be shown to users, as it should be handled internally - \
				 if you see this message in your terminal you've found a bug)",
			),
			Self::PortTooHigh {
				service_name,
				usual_port,
				base_port,
			} => f.write_fmt(format_args!(
				"the {service_name} service usually runs on port {usual_port}, but the base port \
				 was set to {base_port}, which makes the effective port value overflow (the \
				 effective port can be at most {}, rerun with a smaller \"--base-port\" value to \
				 enable this service)",
				u16::MAX
			)),
			Self::Other(e) => e.fmt(f),
		}
	}
}

pub trait SimpleService {
	fn tcp(_: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		Result::<NoFuture, _>::Err(ServiceErr::NoHandler)
	}

	fn udp(_: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		Result::<NoFuture, _>::Err(ServiceErr::NoHandler)
	}
}

macro_rules! service {
	(if $($feature:literal)||+serve $name:ident($cfg:ident)) => {
		#[cfg(any($(feature = $feature),+))]
		{
			use $name::Service;

			let tcp = Service::tcp($cfg);
			let udp = Service::udp($cfg);

			match tcp {
				Ok(service) => {
					::async_std::task::spawn(service);
				}
				Err(ServiceErr::NoHandler) => (),
				Err(e) => {
					::log::error!("{}", e);
				}
			}

			match udp {
				Ok(service) => {
					::async_std::task::spawn(service);
				}
				Err(ServiceErr::NoHandler) => (),
				Err(e) => {
					::log::error!("{}", e);
				}
			}
		}
	};
}

pub fn spawn_all(args: Arguments) {
	let config = Config::from_args(args).expect("argument parsing");

	if config.base_port > 0 {
		info!("Increasing all port numbers by {}", config.base_port);
	}

	service!(if "active" serve active(config));
	service!(if "chargen" serve chargen(config));
	service!(if "daytime" serve daytime(config));
	service!(if "discard" serve discard(config));
	service!(if "echo" serve echo(config));
	service!(if "gopher" serve gopher(config));
	service!(if "message-1" || "message-2" serve message(config));
	service!(if "qotd" serve qotd(config));
	service!(if "time" serve time(config));
}
