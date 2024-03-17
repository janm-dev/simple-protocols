use std::{
	io::{ErrorKind, Read, Write},
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream, UdpSocket},
	thread,
	time::Duration,
};

/// "One popular pattern is 72 character lines of the ASCII printing characters.
/// There are 95 printing characters in the ASCII character set. Sort the
/// characters into an ordered sequence and number the characters from 0 through
/// 94. Think of the sequence as a ring so that character number 0 follows
/// character number 94. On the first line (line 0) put the characters numbered
/// 0 through 71. On the next line (line 1) put the characters numbered 1
/// through 72. And so on. On line N, put characters (0+N mod 95) through (71+N
/// mod 95). End each line with carriage return and line feed."
const DATA: &[u8; 1024 * 3] = b"\
!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefgh\r\n\
\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghi\r\n\
#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghij\r\n\
$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijk\r\n\
%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijkl\r\n\
&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklm\r\n\
'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmn\r\n\
()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmno\r\n\
)*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnop\r\n\
*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopq\r\n\
+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqr\r\n\
,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrs\r\n\
-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrst\r\n\
./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstu\r\n\
/0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuv\r\n\
0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvw\r\n\
123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwx\r\n\
23456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxy\r\n\
3456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz\r\n\
456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{\r\n\
56789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|\r\n\
6789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}\r\n\
789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~\r\n\
89:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ \r\n\
9:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\r\n\
:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"\r\n\
;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#\r\n\
<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$\r\n\
=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%\r\n\
>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&\r\n\
?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'\r\n\
@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'(\r\n\
ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()\r\n\
BCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*\r\n\
CDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+\r\n\
DEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,\r\n\
EFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,-\r\n\
FGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,-.\r\n\
GHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,-./\r\n\
HIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,-./0\r\n\
IJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~ !\"#$%&'()*+,-./01\r\n\
JKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmno\
";

#[test]
fn main() {
	thread::scope(|s| {
		s.spawn(|| tcp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| tcp(IpAddr::V6(Ipv6Addr::LOCALHOST)));

		s.spawn(|| udp(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| udp(IpAddr::V6(Ipv6Addr::LOCALHOST)));
	});
}

/// ["TCP Based Character Generator Service"](https://datatracker.ietf.org/doc/html/rfc864)
fn tcp(ip: IpAddr) {
	// "A server listens for TCP connections on TCP port 19."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 19), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024 * 3];

	// "Once a connection is established a stream of data is sent out the
	// connection"
	tcp.read_exact(&mut buf).unwrap();
	assert_eq!(buf, DATA);

	// "(and any data received is thrown away)."
	write!(tcp, "Hello, World!").unwrap();

	// "This continues until the calling user terminates the connection."
	for _ in 0..2 {
		tcp.read_exact(&mut buf).unwrap();
		let line = buf
			.iter()
			.copied()
			.skip_while(|&b| b != b'\n')
			.skip(1)
			.take_while(|&b| b != b'\r')
			.collect::<Vec<u8>>();
		// "One popular pattern is 72 character lines" ...
		assert_eq!(line.len(), 72);
		// ... "of the ASCII printing characters."
		assert!(line.is_ascii());
	}
}

/// ["UDP Based Character Generator Service"](https://datatracker.ietf.org/doc/html/rfc864)
fn udp(ip: IpAddr) {
	let udp = UdpSocket::bind(if ip.is_ipv4() {
		SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)
	} else {
		SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0)
	})
	.unwrap();

	udp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "A server listens for UDP datagrams on UDP port 19."
	udp.connect(SocketAddr::new(ip, 19)).unwrap();

	// "When a datagram is received" ...
	udp.send(b"Hello, World!").unwrap();

	// ... "an answering datagram is sent" ...
	let n = udp.recv(&mut buf).unwrap();

	// ... "containing a random number (between 0 and 512)" ...
	assert!(n > 0);
	assert!(n < 512);

	// ... "of characters" ...
	assert!(buf[..n].is_ascii());
	assert!(DATA.starts_with(&buf[..n]));

	// ... "(the data in the received datagram is ignored)."
	assert_ne!(&buf[..n], b"Hello, World!");

	// "The service only send one datagram in response to each received datagram"
	assert!(
		matches!(udp.recv(&mut buf[..n]), Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut)
	);
}
