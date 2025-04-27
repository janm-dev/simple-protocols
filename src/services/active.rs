//! The Active Users Protocol ([RFC 865](https://datatracker.ietf.org/doc/html/rfc866))

use std::net::SocketAddr;

use const_str::split;
use log::{info, warn};
use rand::{Rng, seq::IndexedRandom};
use smol::{
	channel::{self, Sender},
	io::AsyncWriteExt,
	net::TcpStream,
	spawn,
};

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 11;

#[allow(long_running_const_eval)]
const USERNAMES: &[&str] = &split!(
	include_str!(concat!(env!("OUT_DIR"), "/usernames.txt")),
	"\n"
);
const USERNAME_END: &[u8] = b"\r\n";

pub struct Service;

impl SimpleService for Service {
	fn tcp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "active",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting active service on TCP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New active users connection from {}",
					FmtMaybeAddr(&incoming.peer_addr())
				);
				spawn(handle_tcp(incoming)).detach();
			}
		})
	}

	fn udp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "active",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting active service on UDP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			UdpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming: (Vec<u8>, SocketAddr, Sender<Vec<u8>>) =
					receiver.recv().await.expect("UDP channel closed");
				info!("New active users datagram from {}", incoming.1);
				spawn(handle_udp(incoming)).detach();
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let usernames = USERNAMES.choose_multiple(&mut rand::rng(), rand::rng().random_range(5..500));

	let mut buf = Vec::with_capacity(512);
	for username in usernames {
		buf.extend(username.as_bytes());
		buf.extend(USERNAME_END);
	}

	if let Err(e) = stream.write_all(&buf).await {
		warn!("error writing data: {e}")
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((_, _, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	let usernames = USERNAMES.choose_multiple(&mut rand::rng(), rand::rng().random_range(5..500));

	let mut buf = Vec::with_capacity(512);
	for username in usernames {
		buf.extend(username.as_bytes());
		buf.extend(USERNAME_END);
	}

	if reply.send(buf).await.is_err() {
		warn!("UDP channel closed");
	};
}
