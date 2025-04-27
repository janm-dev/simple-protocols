//! UDP listeners

use std::{
	net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket as StdSocket},
	sync::Arc,
};

use anyhow::Error;
use log::{debug, trace, warn};
use smol::{
	channel::{self, Sender},
	net::UdpSocket,
	spawn, Async,
};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

use crate::utils::FmtAsciiIsh;

const BUF_SIZE: usize = 1024;

pub struct Listener {
	socket: UdpSocket,
	channel: Sender<(Vec<u8>, SocketAddr, Sender<Vec<u8>>)>,
}

impl Listener {
	pub async fn spawn(
		port: u16,
		channel: Sender<(Vec<u8>, SocketAddr, Sender<Vec<u8>>)>,
	) -> Result<(), Error> {
		let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
		socket.set_nonblocking(true)?;
		socket.bind(&SockAddr::from(SocketAddrV4::new(
			Ipv4Addr::UNSPECIFIED,
			port,
		)))?;

		let listener = UdpSocket::from(Async::new_nonblocking(StdSocket::from(socket))?);
		let listener_v4 = Self {
			socket: listener,
			channel: channel.clone(),
		};

		let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
		socket.set_nonblocking(true)?;
		socket.set_only_v6(true)?;
		socket.bind(&SockAddr::from(SocketAddrV6::new(
			Ipv6Addr::UNSPECIFIED,
			port,
			0,
			0,
		)))?;

		let listener = UdpSocket::from(Async::new_nonblocking(StdSocket::from(socket))?);
		let listener_v6 = Self {
			socket: listener,
			channel,
		};

		spawn(Arc::new(listener_v4).listen()).detach();
		spawn(Arc::new(listener_v6).listen()).detach();

		Ok(())
	}

	async fn listen(self: Arc<Self>) -> ! {
		loop {
			let mut buf = vec![0; BUF_SIZE];

			let (n, addr) = match self.socket.recv_from(&mut buf).await {
				Ok((stream, addr)) => (stream, addr),
				Err(e) => {
					warn!("UDP `recv` error: {e}");
					continue;
				}
			};

			let local_addr = self
				.socket
				.local_addr()
				.expect("unknown local socket address");

			debug!("New datagram {addr} -> {local_addr}");
			trace!(
				"Received {addr} -> {local_addr}: \"{}\"",
				FmtAsciiIsh(&buf[..n])
			);

			buf.truncate(n);
			let (sender, receiver) = channel::unbounded::<Vec<_>>();
			let res = (buf, addr, sender);
			self.channel.send(res).await.expect("UDP channel closed");

			let arc_self = Arc::clone(&self);
			spawn(async move {
				loop {
					if let Ok(buf) = receiver.recv().await {
						trace!("Sending {local_addr} -> {addr}: \"{}\"", FmtAsciiIsh(&buf));

						if let Err(e) = arc_self.socket.send_to(&buf, addr).await {
							warn!("UDP `send` error: {e}");
						};
					} else {
						debug!("End of UDP responses from {local_addr} to {addr}");
						break;
					}
				}
			})
			.detach();
		}
	}
}
