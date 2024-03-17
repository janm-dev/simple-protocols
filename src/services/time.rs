//! The Time Protocol ([RFC 868](https://datatracker.ietf.org/doc/html/rfc868))

use std::net::SocketAddr;

use async_std::{
	channel::{self, Sender},
	io::WriteExt,
	net::TcpStream,
	task::spawn,
};
use log::{info, warn};
use time::OffsetDateTime;

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 37;

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
					"New time connection from {}",
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
				info!("New time datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

const UNIX_EPOCH_OFFSET: i64 = 2_208_988_800;

async fn handle_tcp(mut stream: TcpStream) {
	let now = (OffsetDateTime::now_utc().unix_timestamp() + UNIX_EPOCH_OFFSET) as u32;

	if let Err(e) = stream.write_all(&now.to_be_bytes()).await {
		warn!("error writing data: {e}")
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((_, _, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	let now = (OffsetDateTime::now_utc().unix_timestamp() + UNIX_EPOCH_OFFSET) as u32;

	if reply.send(now.to_be_bytes().to_vec()).await.is_err() {
		warn!("UDP channel closed");
	};
}
