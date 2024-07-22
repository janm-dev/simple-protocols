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
	if message.is_empty() {
		Err("message is empty")?;
	}

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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::utils::FmtAsciiIsh;

	#[test]
	fn parse() {
		const TEST_CASES: &[(&[u8], Result<Message, &str>)] = &[
			(
				b"chris\0\0Hi\r\nHow about lunch?\0",
				Ok(Message::A {
					username: b"chris",
					terminal: b"",
					message: b"Hi\r\nHow about lunch?",
				}),
			),
			(
				b"\x12\x34\x56\0\x78\x90\0\xab\xcd\xef\0",
				Ok(Message::A {
					username: b"\x12\x34\x56",
					terminal: b"\x78\x90",
					message: b"\xab\xcd\xef",
				}),
			),
			(
				b"\0\0\0",
				Ok(Message::A {
					username: b"",
					terminal: b"",
					message: b"",
				}),
			),
			(
				b"chris\0\0Hi\r\nHow about lunch?\0sandy\0console\0910806121325\0\0",
				Err("extra data"),
			),
			(
				b"chris\0\0Hi\r\nHow about lunch?\0chris\0\0Hi\r\nHow about lunch?\0",
				Err("extra data"),
			),
			(b"chris\0\0Hi\r\nHow about lunch?", Err("null")),
			(b"chris\0", Err("missing message")),
			(b"chris", Err("missing terminal")),
			(b"", Err("empty")),
		];

		for (msg, res) in TEST_CASES.iter().cloned() {
			match (super::parse(msg), res) {
				(Ok(parsed), Ok(res)) => assert_eq!(
					parsed,
					res,
					"message parsed incorrectly: parsed {:?} as {parsed:?}, but expected {res:?}",
					FmtAsciiIsh(msg),
				),
				(Err(err), Err(res)) => assert!(
					err.contains(res),
					"message parsing failed incorrectly: got error {err:?}, but expected error \
					 containing {res:?}",
				),
				(Ok(parsed), Err(res)) => panic!(
					"message parsing succeeded unexpectedly: parsed {:?} as {parsed:?}, but \
					 expected error containing {res}",
					FmtAsciiIsh(msg)
				),
				(Err(err), Ok(res)) => {
					panic!("message parsing failed unexpectedly: expected {res:?}, but got {err:?}",)
				}
			}
		}
	}
}
