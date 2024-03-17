use std::{
	io::{Read, Write},
	net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
	str, thread,
	time::Duration,
};

#[test]
fn main() {
	thread::scope(|s| {
		s.spawn(|| tcp_file(IpAddr::V4(Ipv4Addr::LOCALHOST)));
		s.spawn(|| tcp_dir(IpAddr::V4(Ipv4Addr::LOCALHOST)));

		s.spawn(|| tcp_file(IpAddr::V6(Ipv6Addr::LOCALHOST)));
		s.spawn(|| tcp_dir(IpAddr::V6(Ipv6Addr::LOCALHOST)));
	});
}

/// ["The Internet Gopher Protocol"](https://datatracker.ietf.org/doc/html/rfc1436)
fn tcp_file(ip: IpAddr) {
	// "This protocol assumes a reliable data stream; TCP is assumed."
	// "Gopher servers should listen on port 70 (port 70 is assigned to Internet
	// Gopher by IANA)."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 70), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = Vec::new();

	// "Users run client software on their desktop systems, connecting to a server
	// and sending the server a selector (a line of text, which may be empty) via a
	// TCP connection at a well-known port."
	// "The basic server uses the string it gets up to but not including a CR-LF or
	// a TAB, whichever comes first."
	write!(tcp, "/src/services/gopher.rs\tthis should be ignored\r\n").unwrap();

	// "The server responds with a block of text terminated by a period on a line by
	// itself"
	let _ = tcp.read_to_end(&mut buf).unwrap();
	assert!(buf.ends_with(b".\r\n"));
	assert!(buf.strip_suffix(b".\r\n").unwrap() == include_bytes!("../src/services/gopher.rs"));
}

/// ["The Internet Gopher Protocol"](https://datatracker.ietf.org/doc/html/rfc1436)
fn tcp_dir(ip: IpAddr) {
	// "This protocol assumes a reliable data stream; TCP is assumed."
	// "Gopher servers should listen on port 70 (port 70 is assigned to Internet
	// Gopher by IANA)."
	let mut tcp =
		TcpStream::connect_timeout(&SocketAddr::new(ip, 70), Duration::from_secs(1)).unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = Vec::new();

	// "Users run client software on their desktop systems, connecting to a server
	// and sending the server a selector (a line of text, which may be empty) via a
	// TCP connection at a well-known port."
	// "The basic server uses the string it gets up to but not including a CR-LF or
	// a TAB, whichever comes first."
	write!(tcp, "/tests\tthis should be ignored\r\n").unwrap();

	// "The server responds with a block of text terminated by a period on a line by
	// itself"
	let _ = tcp.read_to_end(&mut buf).unwrap();
	assert!(buf.ends_with(b".\r\n"));
	assert!(buf.is_ascii());

	// An empty selector line (as above) "[means] 'list what you have'", to
	// which the reply is a "menu entity" ("Menu" in Paul's NQBNF)
	// "Menu ::= {DirEntity} Lastline.", "Lastline ::= '.'CR-LF."
	let menu = buf.strip_suffix(b"\r\n.\r\n").unwrap();

	// All of the components of DirEntity are made up of ASCII (in the NQBNF)
	let menu = str::from_utf8(menu).unwrap();

	// "DirEntity ::= Type User_Name Tab Selector Tab Host Tab Port CR-LF
	// {RedType User_Name Tab Selector Tab Host Tab Port CR-LF}"
	let dir_entities = menu.split("\r\n");
	for dir_entity in dir_entities {
		// "Type ::= UNASCII.", "UNASCII ::= ASCII - [Tab CR-LF NUL]."
		assert!(dir_entity[..1].is_ascii());
		assert_ne!(&dir_entity[..1], "\t");
		assert_ne!(&dir_entity[..1], "\r\n");
		assert_ne!(&dir_entity[..1], "\0");

		// "User_Name ::= {UNASCII}."
		let section_start = 1;
		let tab_index = dir_entity[section_start..]
			.chars()
			.position(|c| c == '\t')
			.unwrap() + section_start;
		let user_name = &dir_entity[section_start..tab_index];
		assert!(user_name.is_ascii());
		assert!(!user_name.contains('\t'));
		assert!(!user_name.contains("\r\n"));
		assert!(!user_name.contains('\0'));

		// "Selector ::= {UNASCII}."
		let section_start = tab_index + 1;
		let tab_index = dir_entity[section_start..]
			.chars()
			.position(|c| c == '\t')
			.unwrap() + section_start;
		let selector = &dir_entity[section_start..tab_index];
		assert!(selector.is_ascii());
		assert!(!selector.contains('\t'));
		assert!(!selector.contains("\r\n"));
		assert!(!selector.contains('\0'));

		// "Host ::= {{UNASCII - ['.']} '.'} {UNASCII - ['.']}."
		let section_start = tab_index + 1;
		let tab_index = dir_entity[section_start..]
			.chars()
			.position(|c| c == '\t')
			.unwrap() + section_start;
		let host = &dir_entity[section_start..tab_index];
		for host_part in host.split('.') {
			assert!(host_part.is_ascii());
			assert!(!host_part.contains('\t'));
			assert!(!host_part.contains("\r\n"));
			assert!(!host_part.contains('\0'));
			assert!(!host_part.contains('.'));
		}

		// "Port ::= DigitSeq.", "DigitSeq ::= digit {digit}."
		let section_start = tab_index + 1;
		let port = &dir_entity[section_start..];
		assert!(!port.is_empty());
		assert!(port.chars().all(|c| c.is_ascii_digit()));
	}
}
