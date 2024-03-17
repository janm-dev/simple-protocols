//! The Message Send Protocol ([RFC 1159](https://datatracker.ietf.org/doc/html/rfc1159) and [RFC 1312](https://datatracker.ietf.org/doc/html/rfc1312))

#[cfg(feature = "message-1")]
mod v1;
#[cfg(feature = "message-2")]
mod v2;

use std::{
	borrow::Cow,
	fmt::{Display, Formatter, Result as FmtResult},
	net::SocketAddr,
};

use async_std::{
	channel::{self, Sender},
	io::WriteExt,
	net::TcpStream,
	task::spawn,
};
use futures::AsyncReadExt;
use log::{info, warn};

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::{FmtMaybeAddr, FmtMaybeUtf8},
};

pub const PORT: u16 = 18;

pub struct Service;

impl SimpleService for Service {
	fn tcp(_: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		Ok(async {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(PORT, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New Message Send connection from {}",
					FmtMaybeAddr(&incoming.peer_addr())
				);
				spawn(handle_tcp(incoming));
			}
		})
	}

	fn udp(_: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		Ok(async {
			let (sender, receiver) = channel::unbounded();

			UdpListener::spawn(PORT, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("UDP channel closed");
				info!("New Message Send datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

#[derive(Debug, Clone)]
enum Message<'a> {
	#[cfg(feature = "message-1")]
	A {
		username: &'a [u8],
		terminal: &'a [u8],
		message: &'a [u8],
	},
	#[cfg(feature = "message-2")]
	B {
		recipient: Cow<'a, str>,
		recip_term: Cow<'a, str>,
		message: Cow<'a, str>,
		sender: Cow<'a, str>,
		sender_term: Cow<'a, str>,
		cookie: Cow<'a, str>,
		signature: Cow<'a, str>,
	},
}

impl<'a> Display for Message<'a> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			#[cfg(feature = "message-1")]
			Message::A {
				username,
				terminal,
				message,
			} => write!(
				f,
				"to '{}' at '{}': '{}'",
				FmtMaybeUtf8(username),
				FmtMaybeUtf8(terminal),
				FmtMaybeUtf8(message)
			),
			#[cfg(feature = "message-2")]
			Message::B {
				recipient,
				recip_term,
				message,
				sender,
				sender_term,
				cookie,
				signature,
			} => write!(
				f,
				"to '{recipient}' at '{recip_term}': '{message}' from '{sender}' at \
				 '{sender_term}' (with cookie '{cookie}', signed '{signature}')"
			),
		}
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let mut buf = [0; 512];

	loop {
		let bytes = match stream.read(&mut buf).await {
			Ok(0) => break,
			Ok(bytes) => {
				info!(
					"Received {bytes} bytes of message data from {}",
					FmtMaybeAddr(&stream.peer_addr())
				);
				bytes
			}
			Err(e) => {
				warn!("error reading data: {e}");
				break;
			}
		};

		let (msg, reply) = match buf[..bytes].first() {
			#[cfg(feature = "message-1")]
			Some(b'A') => v1::handle_tcp(&buf[1..bytes]),
			#[cfg(feature = "message-2")]
			Some(b'B') => v2::handle_tcp(&buf[1..bytes]),
			Some(_) => (Err("invalid protocol version"), None),
			None => (Err("empty data"), None),
		};

		match msg {
			Ok(msg) => {
				info!("new message received {msg}");

				if let Some(reply) = reply {
					if let Err(e) = stream.write_all(&reply).await {
						warn!("error writing data: {e}")
					}
				}
			}
			Err(err) => {
				warn!("error handling message: {err}");

				if let Some(reply) = reply {
					if let Err(e) = stream.write_all(&reply).await {
						warn!("error writing data: {e}")
					}
				}
			}
		}
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((data, addr, replier): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	info!("Received {} bytes of message data from {addr}", data.len());

	let (msg, reply) = match data.first() {
		#[cfg(feature = "message-1")]
		Some(b'A') => v1::handle_udp(&data),
		#[cfg(feature = "message-2")]
		Some(b'B') => v2::handle_udp(&data),
		Some(_) => (Err("invalid protocol version"), None),
		None => (Err("empty data"), None),
	};

	match msg {
		Ok(msg) => {
			info!("new message received {msg}");

			if let Some(reply) = reply {
				if replier.send(reply.into_owned()).await.is_err() {
					warn!("UDP channel closed");
				};
			}
		}
		Err(err) => {
			warn!("error handling message: {err}");

			if let Some(reply) = reply {
				if replier.send(reply.into_owned()).await.is_err() {
					warn!("UDP channel closed");
				};
			}
		}
	}
}
