//! The Echo Protocol ([RFC 862](https://datatracker.ietf.org/doc/html/rfc862))

use std::net::SocketAddr;

use futures::AsyncReadExt;
use log::{info, warn};
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

pub const PORT: u16 = 7;

pub struct Service;

impl SimpleService for Service {
	fn tcp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "echo",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting echo service on TCP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New Echo connection from {}",
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
				service_name: "echo",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting echo service on UDP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			UdpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("UDP channel closed");
				info!("New Echo datagram from {}", incoming.1);
				spawn(handle_udp(incoming)).detach();
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
