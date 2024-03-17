use std::{
	io::{ErrorKind, Read, Write},
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket},
	thread,
	time::Duration,
};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[test]
fn main() {
	thread::scope(|s| {
		s.spawn(|| tcp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| tcp(IpAddr::V6(Ipv6Addr::LOCALHOST)));

		s.spawn(|| udp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| udp(IpAddr::V6(Ipv6Addr::LOCALHOST)));
	});
}

/// ["TCP Based Daytime Service"](https://datatracker.ietf.org/doc/html/rfc867)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 13."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 13), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "Once a connection is established the current date and time is sent out the
	// connection as a ascii character string" ...
	let n = tcp.read(&mut buf).unwrap();
	let now = OffsetDateTime::now_utc();

	// ... "(and any data received is thrown away)."
	write!(tcp, "Hello, World!").unwrap();
	assert!(
		matches!(tcp.read(&mut buf), Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::ConnectionAborted || e.kind() == ErrorKind::ConnectionReset)
	);

	// "The service closes the connection after sending the quote [sic]."
	thread::sleep(Duration::from_secs(1));
	assert!(
		matches!(write!(tcp, "Hello, World!"), Err(e) if e.kind() == ErrorKind::ConnectionAborted || e.kind() == ErrorKind::ConnectionReset)
	);

	// "There is no specific syntax for the daytime.", but this project uses RFC3339
	// with timezone UTC (with 'Z')
	let datetime =
		String::from_utf8(buf[..n].to_vec()).expect("an RFC3339 datetime is valid UTF-8");
	assert_eq!(datetime.chars().last(), Some('Z'));
	let datetime =
		OffsetDateTime::parse(&datetime, &Rfc3339).expect("the datetime should be RFC3339");
	assert!(datetime.offset().is_utc());
	assert!((now - datetime).unsigned_abs() < Duration::from_secs(1));

	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed."
	assert!(buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));

	// "The daytime should be just one line."
	assert!(!buf.contains(&b'\n'));
}

/// ["UDP Based Daytime Service"](https://datatracker.ietf.org/doc/html/rfc867)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "A server listens for UDP datagrams on UDP port 13."
	udp.connect(SocketAddr::new(ip, 13)).unwrap();

	// "When a datagram is received" ...
	udp.send(b"Hello, World!").unwrap();

	// ... "an answering datagram is sent" ...
	let n = udp.recv(&mut buf).unwrap();
	let now = OffsetDateTime::now_utc();

	// ... "containing the current date and time as a ASCII character string" ...
	// "It is recommended that it be limited to the ASCII printing characters,
	// space, carriage return, and line feed."
	assert!(buf[..n]
		.iter()
		.all(|c| c.is_ascii_graphic() || b" \r\n".contains(c)));

	// "There is no specific syntax for the daytime.", but this project uses RFC3339
	// with timezone UTC (with 'Z')
	let datetime =
		String::from_utf8(buf[..n].to_vec()).expect("an RFC3339 datetime is valid UTF-8");
	assert_eq!(datetime.chars().last(), Some('Z'));
	let datetime =
		OffsetDateTime::parse(&datetime, &Rfc3339).expect("the datetime should be RFC3339");
	assert!(datetime.offset().is_utc());
	assert!((now - datetime).unsigned_abs() < Duration::from_secs(1));

	// ... "(the data in the received datagram is ignored)."
	assert_ne!(&buf[..n], b"Hello, World!");
}
