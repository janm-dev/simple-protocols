use std::{
	io::Read,
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket},
	thread,
	time::Duration,
};

use time::OffsetDateTime;

#[test]
fn main() {
	thread::scope(|s| {
		s.spawn(|| tcp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| tcp(IpAddr::V6(Ipv6Addr::LOCALHOST)));

		s.spawn(|| udp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| udp(IpAddr::V6(Ipv6Addr::LOCALHOST)));
	});
}

/// ["via TCP"](https://datatracker.ietf.org/doc/html/rfc868)
fn tcp(ip: IpAddr) {
	// "S: Listen on port 37 (45 octal).", "U: Connect to port 37."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 37), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "S: Send the time as a 32 bit binary number.", "U: Receive the time."
	let n = tcp.read(&mut buf).unwrap();
	let now = OffsetDateTime::now_utc();

	// "The time is the number of seconds since 00:00 (midnight) 1 January 1900 GMT,
	// such that the time 1 is 12:00:01 am on 1 January 1900 GMT;" ...
	let time = u32::from_be_bytes(buf[..n].try_into().unwrap());
	// ("the time 2,208,988,800 corresponds to 00:00 1 Jan 1970 GMT")
	let time = OffsetDateTime::from_unix_timestamp(time as i64 - 2_208_988_800).unwrap();
	assert!((time - now).unsigned_abs() < Duration::from_secs(1));

	// ... "this base will serve until the year 2036."
	if now.year() >= 2036 {
		panic!("the time protocol expired");
	}
}

/// ["via UDP"](https://datatracker.ietf.org/doc/html/rfc868)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "S: Listen on port 37 (45 octal).", "U: Send an empty datagram to port 37."
	udp.connect(SocketAddr::new(ip, 37)).unwrap();

	// "S: Receive the empty datagram."
	udp.send(b"TODO: this should be empty, but that doesn't work when the server is running on linux (?)").unwrap();

	// "S: Send a datagram containing the time as a 32 bit binary number.", "U:
	// Receive the time datagram."
	let n = udp.recv(&mut buf).unwrap();
	let now = OffsetDateTime::now_utc();

	// "The time is the number of seconds since 00:00 (midnight) 1 January 1900 GMT,
	// such that the time 1 is 12:00:01 am on 1 January 1900 GMT;" ...
	let time = u32::from_be_bytes(buf[..n].try_into().unwrap());
	// ("the time 2,208,988,800 corresponds to 00:00 1 Jan 1970 GMT")
	let time = OffsetDateTime::from_unix_timestamp(time as i64 - 2_208_988_800).unwrap();
	assert!((time - now).unsigned_abs() < Duration::from_secs(1));

	// ... "this base will serve until the year 2036."
	if now.year() >= 2036 {
		panic!("the time protocol expired");
	}
}
