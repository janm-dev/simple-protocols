//! The Echo Protocol ([RFC 862](https://datatracker.ietf.org/doc/html/rfc862))

use std::net::SocketAddr;

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
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 7;

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
					"New Echo connection from {}",
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
				info!("New Echo datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let mut buf = [0; 512];

	loop {
		let bytes = match stream.read(&mut buf).await {
			Ok(0) => break,
			Ok(bytes) => {
				info!(
					"Echoing {bytes} bytes of data back to {}",
					FmtMaybeAddr(&stream.peer_addr())
				);
				bytes
			}
			Err(e) => {
				warn!("error reading data: {e}");
				break;
			}
		};

		if let Err(e) = stream.write_all(&buf[..bytes]).await {
			warn!("error writing data: {e}")
		}
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((data, addr, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	info!("Echoing {} bytes of data from {addr}", data.len());

	if reply.send(data).await.is_err() {
		warn!("UDP channel closed");
	};
}
