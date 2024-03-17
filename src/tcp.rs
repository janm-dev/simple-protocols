//! TCP listeners

use std::{
	ffi::c_int,
	net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener as StdListener},
};

use anyhow::Error;
use async_std::{
	channel::Sender,
	net::{TcpListener, TcpStream},
	task::spawn,
};
use log::{debug, warn};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

const TCP_BACKLOG: c_int = 1024;

pub struct Listener {
	listener: TcpListener,
	channel: Sender<TcpStream>,
}

impl Listener {
	pub async fn spawn(port: u16, channel: Sender<TcpStream>) -> Result<(), Error> {
		let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;
		socket.set_nodelay(true)?;
		socket.set_nonblocking(true)?;
		socket.bind(&SockAddr::from(SocketAddrV4::new(
			Ipv4Addr::UNSPECIFIED,
			port,
		)))?;
		socket.listen(TCP_BACKLOG)?;

		let listener = TcpListener::from(StdListener::from(socket));
		let listener_v4 = Self {
			listener,
			channel: channel.clone(),
		};

		let socket = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP))?;
		socket.set_nodelay(true)?;
		socket.set_nonblocking(true)?;
		socket.set_only_v6(true)?;
		socket.bind(&SockAddr::from(SocketAddrV6::new(
			Ipv6Addr::UNSPECIFIED,
			port,
			0,
			0,
		)))?;
		socket.listen(TCP_BACKLOG)?;

		let listener = TcpListener::from(StdListener::from(socket));
		let listener_v6 = Self { listener, channel };

		spawn(listener_v4.listen());
		spawn(listener_v6.listen());

		Ok(())
	}

	async fn listen(self) -> ! {
		loop {
			let (stream, addr) = match self.listener.accept().await {
				Ok((stream, addr)) => (stream, addr),
				Err(e) => {
					warn!("TCP `accept` error: {e}");
					continue;
				}
			};

			debug!(
				"New connection {addr} -> {}",
				self.listener
					.local_addr()
					.expect("unknown local socket address")
			);

			self.channel.send(stream).await.expect("TCP channel closed");
		}
	}
}
