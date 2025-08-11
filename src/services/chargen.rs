//! The Character Generator Protocol ([RFC 864](https://datatracker.ietf.org/doc/html/rfc864))

use std::net::SocketAddr;

use log::{info, warn};
use rand::Rng;
use smol::{channel, channel::Sender, io::AsyncWriteExt, net::TcpStream, spawn};

use crate::{
	services::{Config, Future, ServiceErr, ServiceRet, SimpleService},
	tcp::Listener as TcpListener,
	udp::Listener as UdpListener,
	utils::FmtMaybeAddr,
};

pub const PORT: u16 = 19;
const LINE_LEN: usize = 72;
const LINE_END: &[u8] = b"\r\n";
const CHARACTERS: &str = r##"!"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`abcdefghijklmnopqrstuvwxyz{|}~ "##;

pub struct Service;

impl SimpleService for Service {
	fn tcp(config: &'static Config) -> Result<impl Future<Output = ServiceRet>, ServiceErr> {
		let mapped_port = PORT
			.checked_add(config.base_port)
			.ok_or(ServiceErr::PortTooHigh {
				service_name: "chargen",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting chargen service on TCP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			TcpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("TCP channel closed");
				info!(
					"New CHARGEN connection from {}",
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
				service_name: "chargen",
				usual_port: PORT,
				base_port: config.base_port,
			})?;

		info!("starting chargen service on UDP port {mapped_port}");

		Ok(async move {
			let (sender, receiver) = channel::unbounded();

			UdpListener::spawn(mapped_port, sender)
				.await
				.expect("error creating listener");

			loop {
				let incoming = receiver.recv().await.expect("UDP channel closed");
				info!("New CHARGEN datagram from {}", incoming.1);
				spawn(handle_udp(incoming)).detach();
			}
		})
	}
}

async fn handle_tcp(mut stream: TcpStream) {
	const CHARACTERS_2: &[u8] = const_format::concatcp!(CHARACTERS, CHARACTERS).as_bytes();

	let mut buf = [0; LINE_LEN + LINE_END.len()];
	buf[LINE_LEN..].copy_from_slice(LINE_END);

	for i in (0..LINE_LEN).cycle() {
		buf[..LINE_LEN].copy_from_slice(&CHARACTERS_2[i..(i + LINE_LEN)]);

		if let Err(e) = stream.write_all(&buf).await {
			warn!("error writing data: {e}");
			break;
		};
	}

	info!(
		"Connection with {} closing",
		FmtMaybeAddr(&stream.peer_addr())
	);
}

async fn handle_udp((_, _, reply): (Vec<u8>, SocketAddr, Sender<Vec<u8>>)) {
	const CHARACTERS_512: &[u8; 512] = b"\
		!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefgh\r\n\
		\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghi\r\n\
		#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghij\r\n\
		$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijk\r\n\
		%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijkl\r\n\
		&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklm\r\n\
		'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghij\
	";

	let len = rand::rng().random_range(1..512);
	if reply.send(CHARACTERS_512[..len].to_vec()).await.is_err() {
		warn!("UDP channel closed");
	};
}
