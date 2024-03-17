//! The Message Send Protocol, version 1 ([RFC 1159](https://datatracker.ietf.org/doc/html/rfc1159))

use std::borrow::Cow;

use super::Message;

pub fn handle_tcp(data: &[u8]) -> (Result<Message, &'static str>, Option<Cow<'_, [u8]>>) {
	match parse(&data[1..]) {
		Ok(msg) => (Ok(msg), None),
		Err(err) => (Err(err), None),
	}
}

pub fn handle_udp(data: &[u8]) -> (Result<Message, &'static str>, Option<Cow<'_, [u8]>>) {
	match parse(&data[1..]) {
		Ok(msg) => (Ok(msg), Some(Cow::Borrowed(data))),
		Err(err) => (Err(err), None),
	}
}

pub fn parse(message: &[u8]) -> Result<Message, &'static str> {
	let mut parts = message.split(|&b| b == b'\0');

	let Some(username) = parts.next() else {
		Err("missing username")?
	};

	let Some(terminal) = parts.next() else {
		Err("missing terminal name")?
	};

	let Some(message) = parts.next() else {
		Err("missing message")?
	};

	match parts.next() {
		None => Err("no final null terminator")?,
		Some(b"") => (),
		Some(_) => Err("extra data after message")?,
	}

	Ok(Message::A {
		username,
		terminal,
		message,
	})
}
