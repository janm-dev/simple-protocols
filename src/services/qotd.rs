//! The Quote of the Day Protocol ([RFC 865](https://datatracker.ietf.org/doc/html/rfc865))

use std::net::SocketAddr;

use async_std::{
	channel::{self, Sender},
	io::WriteExt,
	net::TcpStream,
	task::spawn,
};
use const_str::split;
use log::{info, warn};
use rand::seq::SliceRandom;

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 17;

#[allow(long_running_const_eval)]
const QUOTES: &[&str] = &split!(include_str!(concat!(env!("OUT_DIR"), "/quotes.txt")), "\n");
const QUOTE_END: &[u8] = b"\r\n";

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
					"New QOTD connection from {}",
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
				info!("New QOTD datagram from {}", incoming.1);
				spawn(handle_udp(incoming));
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	let mut buf = [0; 512];
	let quote = QUOTES
		.choose(&mut rand::thread_rng())
		.expect("there are not quotes")
		.as_bytes();
	buf[..quote.len()].copy_from_slice(quote);
	buf[quote.len()..quote.len() + QUOTE_END.len()].copy_from_slice(QUOTE_END);

	if let Err(e) = stream
		.write_all(&buf[..quote.len() + QUOTE_END.len()])
		.await
	{
		warn!("error writing data: {e}")
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((_, _, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	let mut buf = [0; 512];
	let quote = QUOTES
		.choose(&mut rand::thread_rng())
		.expect("there are not quotes")
		.as_bytes();
	buf[..quote.len()].copy_from_slice(quote);
	buf[quote.len()..quote.len() + QUOTE_END.len()].copy_from_slice(QUOTE_END);

	if reply
		.send(buf[..quote.len() + QUOTE_END.len()].to_vec())
		.await
		.is_err()
	{
		warn!("UDP channel closed");
	};
}
