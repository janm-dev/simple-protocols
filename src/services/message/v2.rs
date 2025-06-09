//! The Message Send Protocol, version 2 ([RFC 1312](https://datatracker.ietf.org/doc/html/rfc1312))

use std::borrow::Cow;

use super::Message;
use crate::utils::decode_iso_8859_1;

pub fn handle_tcp(data: &[u8]) -> (Result<Message<'_>, &'static str>, Option<Cow<'_, [u8]>>) {
	match parse(&data[1..]) {
		Ok(msg) => (Ok(msg), Some(Cow::Borrowed(b"+\0"))),
		Err(err) => (Err(err), Some(Cow::Owned(format!("-{err}\0").into_bytes()))),
	}
}

pub fn handle_udp(data: &[u8]) -> (Result<Message<'_>, &'static str>, Option<Cow<'_, [u8]>>) {
	match parse(&data[1..]) {
		Ok(msg) => {
			let reply = if matches!(&msg, Message::B { recipient, .. } if !recipient.is_empty()) {
				Some(Cow::Borrowed(&b"+\0"[..]))
			} else {
				None
			};

			(Ok(msg), reply)
		}
		Err(err) => (Err(err), None),
	}
}

pub fn parse(message: &[u8]) -> Result<Message<'_>, &'static str> {
	let mut parts = message.split(|&b| b == b'\0');

	let recipient = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(recipient)) => recipient,
		Some(Err(_)) => Err("error decoding recipient")?,
		None => Err("missing recipient")?,
	};

	let recip_term = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(recip_term)) => recip_term,
		Some(Err(_)) => Err("error decoding recipient terminal name")?,
		None => Err("missing recipient terminal name")?,
	};

	let message = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(message)) => message,
		Some(Err(_)) => Err("error decoding message")?,
		None => Err("missing message")?,
	};

	let sender = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(sender)) => sender,
		Some(Err(_)) => Err("error decoding sender")?,
		None => Err("missing sender")?,
	};

	let sender_term = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(sender_term)) => sender_term,
		Some(Err(_)) => Err("error decoding sender terminal")?,
		None => Err("missing sender terminal")?,
	};

	let cookie = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(cookie)) => cookie,
		Some(Err(_)) => Err("error decoding cookie")?,
		None => Err("missing cookie")?,
	};

	let signature = match parts.next().map(decode_iso_8859_1) {
		Some(Ok(signature)) => signature,
		Some(Err(_)) => Err("error decoding signature")?,
		None => Err("missing signature")?,
	};

	match parts.next() {
		None => Err("no final null terminator")?,
		Some(b"") => (),
		Some(_) => Err("extra data after message")?,
	}

	Ok(Message::B {
		recipient,
		recip_term,
		message,
		sender,
		sender_term,
		cookie,
		signature,
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
				b"chris\0\0Hi\r\nHow about lunch?\0sandy\0console\0910806121325\0\0",
				Ok(Message::B {
					recipient: Cow::Borrowed("chris"),
					recip_term: Cow::Borrowed(""),
					message: Cow::Borrowed("Hi\r\nHow about lunch?"),
					sender: Cow::Borrowed("sandy"),
					sender_term: Cow::Borrowed("console"),
					cookie: Cow::Borrowed("910806121325"),
					signature: Cow::Borrowed(""),
				}),
			),
			(
				b"chris\0\0Hi\r\nHow about lunch?\0sandy\0console\0910806121325\0",
				Err("null"),
			),
			(b"chris\0\0Hi\r\nHow about lunch?\0", Err("missing sender")),
			(
				b"\0\0\0\0\0\0\0",
				Ok(Message::B {
					recipient: Cow::Borrowed(""),
					recip_term: Cow::Borrowed(""),
					message: Cow::Borrowed(""),
					sender: Cow::Borrowed(""),
					sender_term: Cow::Borrowed(""),
					cookie: Cow::Borrowed(""),
					signature: Cow::Borrowed(""),
				}),
			),
			(
				b"\x12\0\x34\0\x56\0\x78\0\x89\0\xab\0\xcd\0",
				Err("error decoding"),
			),
			(
				b"chris\0\0Hi\r\nHow about lunch?\0sandy\0console\0910806121325\0\0chris\0\0Hi\r\nHow about lunch?\0sandy\0console\0910806121325\0\0",
				Err("extra data"),
			),
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
