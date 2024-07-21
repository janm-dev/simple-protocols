//! The Daytime Protocol ([RFC 867](https://datatracker.ietf.org/doc/html/rfc867))

use std::net::SocketAddr;

use async_std::{
	channel::{self, Sender},
	io::WriteExt,
	net::TcpStream,
	task::spawn,
};
use log::{info, warn};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 13;

pub struct Service;

impl SimpleService for Service {
	fn tcp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "daytime",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting daytime service on TCP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New daytime connection from {}",
					FmtMaybeAddr(&incoming.peer_addr())
				);
				spawn(handle_tcp(incoming));
			}
		})
	}

	fn udp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "daytime",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting daytime service on UDP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			UdpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("UDP channel closed");
				info!("New daytime datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let now = OffsetDateTime::now_utc()
		.format(&Rfc3339)
		.expect("RFC3339 format is invalid");

	if let Err(e) = stream.write_all(now.as_bytes()).await {
		warn!("error writing data: {e}")
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((_, _, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	let now = OffsetDateTime::now_utc()
		.format(&Rfc3339)
		.expect("RFC3339 format is invalid");

	if reply.send(now.into_bytes()).await.is_err() {
		warn!("UDP channel closed");
	};
}
