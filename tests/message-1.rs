use std::{
	io::Write,
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

/// ["Message Syntax"](https://datatracker.ietf.org/doc/html/rfc1159)
/// "The message should consist of several parts. The first part is a single
/// octet indicating the protocol revision, currently decimal 65, 'A'. The
/// second part is the name of the user that the message is directed to. This
/// and the remaining parts are null-terminated, and consist of eight-bit
/// characters. Do not strip the eighth bit of the characters. The third part is
/// the name of the terminal. The fourth part is the actual message."
const TEST_MESSAGE: &[u8] =
	b"Auser\0terminal_number_\xff\0hello this is a message to user at terminal number 255\0";

/// ["TCP Based Message Send Service"](https://datatracker.ietf.org/doc/html/rfc1159)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 18."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 18), Duration::from_secs(1)).unwrap();

	// "Once a connection is established a short message is sent by the client out
	// the connection" ...
	tcp.write_all(TEST_MESSAGE).unwrap();

	// ... "(and any data received by the client is thrown away)."

	// "The total length of the message shall be less than 512 octets. This includes
	// all four parts, and any terminating nulls."
	assert!(TEST_MESSAGE.len() < 512);

	// "The client closes the connection after sending the message."
	drop(tcp);
}

/// ["UDP Based Message Send Service"](https://datatracker.ietf.org/doc/html/rfc1159)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "A server listens for UDP datagrams on UDP port 18."
	udp.connect(SocketAddr::new(ip, 18)).unwrap();

	// "When a datagram is received by the server, " ...
	udp.send(TEST_MESSAGE).unwrap();

	// ... "an answering datagram is sent back to the client containing exactly the
	// same data."
	let n = udp.recv(&mut buf).unwrap();
	assert!(&buf[..n] == TEST_MESSAGE);
}
