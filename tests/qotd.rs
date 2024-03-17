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

/// ["TCP Based Character Generator \[sic\] Service"](https://datatracker.ietf.org/doc/html/rfc865)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 17."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 17), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "Once a connection is established a short message is sent out the connection"
	// ...
	let n = tcp.read(&mut buf).unwrap();

	// ... "(and any data received is thrown away)."
	write!(tcp, "Hello, World!").unwrap();
	let res = tcp.read(&mut buf);
	assert!(
		matches!(res, Ok(0))
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::WouldBlock)
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::TimedOut)
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::ConnectionAborted)
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::ConnectionReset)
	);

	// "The service closes the connection after sending the quote."
	thread::sleep(Duration::from_secs(1));
	let res = write!(tcp, "Hello, World!");
	assert!(
		res.is_ok()
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::ConnectionAborted)
			|| matches!(res, Err(ref e) if e.kind() == ErrorKind::ConnectionReset)
	);

	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed."
	assert!(buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));

	// "The quote may be just one or up to several lines, but it should be less than
	// 512 characters."
	assert!(n < 512);
}

/// ["UDP Based Character Generator \[sic\] Service"](https://datatracker.ietf.org/doc/html/rfc865)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
	let mut buf = vec![0; 1024];

	// "A server listens for UDP datagrams on UDP port 17."
	udp.connect(SocketAddr::new(ip, 17)).unwrap();

	// "When a datagram is received" ...
	udp.send(b"Hello, World!").unwrap();

	// ... "an answering datagram is sent" ...
	let n = udp.recv(&mut buf).unwrap();

	// ... "containing a quote" ...
	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed."
	assert!(buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));

	// "The quote may be just one or up to several lines, but it should be less than
	// 512 characters."
	assert!(n < 512);

	// ... "(the data in the received datagram is ignored)."
	assert_ne!(&buf[..n], b"Hello, World!");
}
