use std::{
	io::{Read, Write},
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

/// ["Message Syntax"](https://datatracker.ietf.org/doc/html/rfc1312)
/// "The message consists of several parts, all of which must be present. The
/// first part is a single octet indicating the protocol revision, currently
/// decimal 66, 'B'. The remaining parts are null-terminated sequences of
/// eight-bit characters in the ISO 8859/1 alphabet. Some parts may be empty.
/// All comparisons of parts (e.g., recipient, cookie, etc.) are
/// case-insensitive. The parts are as follows:"
/// - "RECIPIENT - The name of the user that the message is directed to. If this
///   part is empty, the message may be delivered to any user of the destination
///   system."
/// - "RECIP-TERM - The name of the terminal to which the message is to be
///   delivered. The syntax and semantics of terminal names are outside the
///   scope of this specification."
/// - "MESSAGE - The actual message. New lines should be represented using the
///   usual Netascii CR + LF. The message text may only contain printable
///   characters from the ISO 8859/1 set, which is upward compatible from
///   USASCII, plus CR, LF and TAB. No other control codes or escape sequences
///   may be included: the client should strip them from the message before it
///   is transmitted, and the server must check each incoming message for
///   illegal codes."
/// - "SENDER - The username of the sender."
/// - "SENDER-TERM - The name of the sending user's terminal."
/// - "COOKIE - A magic cookie. This part must be present in all messages, but
///   is only of significance for the UDP service. The maximum length of a
///   cookie is 32 octets, excluding the terminating null."
/// - "SIGNATURE - A token which, if present, may be used by the server to
///   verify the identity of the sender."
#[allow(clippy::octal_escapes)] // it's the null byte
const TEST_MESSAGE: &[u8] =
	b"Buser\0t\xe6rminal_255\0hello this is a message to user at t\xe6rminal number 255\0sender\0sender's terminal\01701024822\0some token, idk\0";

/// ["TCP Based Message Send Service"](https://datatracker.ietf.org/doc/html/rfc1312)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 18."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 18), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "The total length of the message shall be less than 512 octets. This includes
	// all eight parts, and any terminating nulls. UDP packets are limited to 512
	// octets."
	assert!(TEST_MESSAGE.len() < 512);

	// "Multiple messages can be sent over the same channel."
	for _ in 0..3 {
		// "Once a connection is established a message is sent by the client over the
		// connection."
		tcp.write_all(TEST_MESSAGE).unwrap();

		// "The server replies" ...
		let n = tcp.read(&mut buf).unwrap();

		// ... "with a single character indicating positive ("+") or negative ("-")
		// acknowledgment, " ...
		assert!(buf[..n][0] == b'+' || buf[..n][0] == b'-');

		// ... "immediately followed by an optional message of explanation, terminated
		// with a null."
		assert!(buf[n] == b'\0');
	}
}

/// ["UDP Based Message Send Service"](https://datatracker.ietf.org/doc/html/rfc1312)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "The total length of the message shall be less than 512 octets. This includes
	// all eight parts, and any terminating nulls. UDP packets are limited to 512
	// octets."
	assert!(TEST_MESSAGE.len() < 512);

	// "A server listens for UDP datagrams on UDP port 18."
	udp.connect(SocketAddr::new(ip, 18)).unwrap();

	// "When a datagram is received by the server, " ...
	udp.send(TEST_MESSAGE).unwrap();

	// ... "an answering datagram may be sent back to the client."
	let n = udp.recv(&mut buf).unwrap();

	// "If the message was addressed to a particular user (i.e., the RECIPIENT part
	// was non-empty) and was successfully delivered to that user, a positive
	// acknowledgement should be sent (as described above). If the message was
	// directed at any user (i.e., the RECIPIENT part is empty), or if the message
	// could not be delivered for some reason, no reply is sent."
	assert!(buf[..n][0] == b'+');
	assert!(buf[n] == b'\0');
}
