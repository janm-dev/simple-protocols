//! The Discard Protocol ([RFC 863](https://datatracker.ietf.org/doc/html/rfc863))

use std::net::SocketAddr;

use async_std::{channel, channel::Sender, net::TcpStream, task::spawn};
use futures::AsyncReadExt;
use log::{info, warn};

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 9;

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
					"New Discard connection from {}",
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
				info!("New Discard datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let mut buf = [0; 512];

	loop {
		match stream.read(&mut buf).await {
			Ok(0) => break,
			Ok(bytes) => info!(
				"Discarding {bytes} bytes of data from {}",
				FmtMaybeAddr(&stream.peer_addr())
			),
			Err(e) => {
				warn!("error reading data: {e}");
				break;
			}
		};
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((data, addr, _): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	info!("Discarding {} bytes of data from {addr}", data.len());
}
