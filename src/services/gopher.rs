//! The Internet Gopher Protocol ([RFC 1436](https://datatracker.ietf.org/doc/html/rfc1436))

use std::{
	borrow::Cow,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	io::Write,
};

use async_std::{
	channel::{self},
	io::WriteExt,
	net::TcpStream,
	task::spawn,
};
use futures::AsyncReadExt;
use log::{debug, info, warn};

use crate::{
	fs::{self, Entry},
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	utils::{FmtAsciiIsh, FmtMaybeAddr},
};

pub const PORT: u16 = 70;

pub struct Service;

impl SimpleService for Service {
	fn tcp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "gopher",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		let hostname = config.hostname.as_ref().ok_or(ServiceErr::MissingConfig {
			service_name: "gopher",
			config_name: "hostname",
		})?;

		info!("starting gopher service on TCP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New Gopher connection from {}",
					FmtMaybeAddr(&incoming.peer_addr())
				);
				spawn(handle(incoming, hostname));
			}
		})
	}
}

#[derive(Debug)]
enum Selected {
	/// An unknown non-empty selector was requested
	Unknown,
	/// The contained file was selected
	File(&'static str),
	/// The contained directory was selected (for the empty selector this is the
	/// root entry)
	Directory(&'static [Entry<'static>]),
}

impl Selected {
	pub fn get(selector: &[u8]) -> Self {
		if selector.is_empty() {
			Self::Directory(fs::root_entries())
		} else if let Ok(entry) = fs::read(selector) {
			match entry {
				Entry::File { contents, .. } => Self::File(contents),
				Entry::Directory { entries, .. } => Self::Directory(entries),
			}
		} else {
			Self::Unknown
		}
	}
}

/// Gopher item types supported by this server
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemType {
	File = b'0',
	Directory = b'1',
	Error = b'3',
}

impl ItemType {
	pub fn for_entry(entry: &Entry<'_>) -> Self {
		match (entry.is_file(), entry.is_directory()) {
			(true, false) => Self::File,
			(false, true) => Self::Directory,
			_ => Self::Error,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Item<'a> {
	pub kind: ItemType,
	pub name: Cow<'a, str>,
	pub selector: Cow<'a, str>,
	pub host: Cow<'a, str>,
	pub port: u16,
}

impl Display for Item<'_> {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		write!(
			f,
			"{}{}\t{}\t{}\t{}\r\n",
			self.kind as u8 as char, self.name, self.selector, self.host, self.port
		)
	}
}

async fn handle(mut stream: TcpStream, hostname: &str) {
	let mut buf = [0u8; 512];
	let mut n = 0;

	while !buf[..n].ends_with(b"\r\n") {
		n += match stream.read(&mut buf[n..]).await {
			Ok(0) => break,
			Ok(n) => n,
			Err(e) => {
				warn!("error reading data: {e}");
				return;
			}
		};
	}

	let mut saw_cr = false;
	let Some(selector_end) = buf[..n].iter().position(|&b| {
		b == b'\t' || saw_cr && b == b'\n' || {
			if b == b'\r' {
				saw_cr = true;
			}
			false
		}
	}) else {
		warn!("error parsing selector line");
		return;
	};

	let selector = &buf[..=selector_end];
	let selector = selector.strip_suffix(b"\r\n").unwrap_or(selector);
	let selector = selector.strip_suffix(b"\t").unwrap_or(selector);
	let selector = if selector == b"/" { b"" } else { selector };

	debug!("Selector is \"{}\"", FmtAsciiIsh(selector));

	let response = Selected::get(selector);
	let mut res = Vec::new();

	let _ = match response {
		Selected::File(contents) => Write::write_fmt(&mut res, format_args!("{contents}.\r\n")),
		Selected::Directory(entries) => {
			for entry in entries {
				let _ = Write::write_fmt(
					&mut res,
					format_args!("{}", Item {
						kind: ItemType::for_entry(entry),
						name: entry.name().into(),
						selector: (String::from_utf8(selector.to_vec())
							.expect("the input was a valid path, so it's also a valid string")
							+ "/" + entry.name())
						.into(),
						host: hostname.into(),
						port: PORT
					}),
				);
			}

			Write::write_all(&mut res, b".\r\n")
		}
		Selected::Unknown => Write::write_fmt(
			&mut res,
			format_args!("{}.\r\n", Item {
				kind: ItemType::Error,
				name: "not found".into(),
				selector: "".into(),
				host: hostname.into(),
				port: PORT
			}),
		),
	};

	if let Err(e) = stream.write_all(&res).await {
		warn!("error writing data: {e}")
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}
