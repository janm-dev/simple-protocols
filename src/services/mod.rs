//! Service protocol implementations

pub use std::future::Future;
use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	pin::Pin,
	task::{Context, Poll},
};

use pico_args::Arguments;

// Declare the modules here because rust-analyzer wasn't too happy with
// declaring them inside of the `service` macro
#[cfg(feature = "discard")]
mod discard;
#[cfg(feature = "echo")]
mod echo;

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
	pub hostname: Option<String>,
}

impl Config {
	pub fn from_args() -> Result<&'static Self, anyhow::Error> {
		let mut args = Arguments::from_env();

		let cfg = Self {
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

pub fn spawn_all() {
	let config = Config::from_args().expect("argument parsing");

	service!(if "discard" serve discard(config));
	service!(if "echo" serve echo(config));
}
