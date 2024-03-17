use std::{
	io::{ErrorKind, Read, Write},
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket},
	thread,
	time::Duration,
};

#[test]
fn main() {
	thread::scope(|s| {
		s.spawn(|| tcp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| tcp(IpAddr::V6(Ipv6Addr::LOCALHOST)));

		s.spawn(|| udp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| udp(IpAddr::V6(Ipv6Addr::LOCALHOST)));
	});
}

/// ["TCP Based Discard Service"](https://datatracker.ietf.org/doc/html/rfc863)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 9."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 9), Duration::from_secs(1)).unwrap();

	let mut buf = vec![0; 1024];
	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();

	// "This continues until the calling user terminates the connection."
	for _ in 0..5 {
		// "Once a connection is established any data received is thrown away."
		writeln!(tcp, "Hello, World!").unwrap();

		// "No response is sent."
		assert!(
			matches!(tcp.read(&mut buf), Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut)
		);

		// Don't waste time in the other iterations of this loop
		tcp.set_read_timeout(Some(Duration::from_millis(10)))
			.unwrap();
	}
}

/// ["UDP Based Discard Service"](https://datatracker.ietf.org/doc/html/rfc863)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "A server listens for UDP datagrams on UDP port 9."
	udp.connect(SocketAddr::new(ip, 9)).unwrap();

	// "When a datagram is received, it is thrown away."
	udp.send(b"Hello, World!").unwrap();

	// "No response is sent."
	assert!(
		matches!(udp.recv(&mut buf), Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut)
	);
}
