use std::{
	io::{ErrorKind, Read},
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

/// ["TCP Based Active Users Service"](https://datatracker.ietf.org/doc/html/rfc866)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 11."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 11), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = Vec::new();

	// "Once a connection is established a list of the currently active users is
	// sent out the connection" ...
	let n = tcp.read_to_end(&mut buf).unwrap();

	// ... "(and any data received is thrown away)."
	assert_ne!(&buf[..n], b"Hello, World!");

	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed. Each user should be listed on a
	// separate line."
	assert!(&buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));
}

/// ["UDP Based Active Users Service"](https://datatracker.ietf.org/doc/html/rfc866)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 16 * 1024];

	// "A server listens for UDP datagrams on UDP port 11."
	udp.connect(SocketAddr::new(ip, 11)).unwrap();

	// "When a datagram is received" ...
	udp.send(b"Hello, World!").unwrap();

	// ... "an answering datagram is sent" ...
	let mut n = udp.recv(&mut buf).unwrap();

	// ... "containing a list of the currently active users (the data in the
	// received datagram is ignored)."
	assert_ne!(&buf[..n], b"Hello, World!");

	// "If the list does not fit in one datagram then send a sequence of datagrams
	// but don't break the information for a user (a line) across a datagram."
	assert!(buf[..n].ends_with(b"\r\n"));
	// "The user side should wait for a timeout for all datagrams to arrive. Note
	// that they are not necessarily in order."
	loop {
		n += match udp.recv(&mut buf[n..]) {
			Ok(n) => n,
			Err(e) if e.kind() == ErrorKind::TimedOut => {
				break;
			}
			res => {
				res.unwrap();
				unreachable!()
			}
		};

		assert!(buf[..n].ends_with(b"\r\n"));
	}

	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed. Each user should be listed on a
	// separate line."
	assert!(buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));
}
