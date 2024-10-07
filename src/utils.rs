//! Non-protocol-related utilities

use std::{
	borrow::Cow,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	net::SocketAddr,
	str,
};

/// Decode an ISO/IES 8859-1 string
pub fn decode_iso_8859_1(s: &[u8]) -> Result<Cow<'_, str>, usize> {
	if s.is_ascii() {
		Ok(Cow::Borrowed(
			str::from_utf8(s).expect("ascii is valid utf-8"),
		))
	} else if s
		.iter()
		.all(|b| (0x20..=0x7e).contains(b) || (0xa0..=0xff).contains(b))
	{
		Ok(Cow::Owned(s.iter().map(|&b| b as char).collect()))
	} else {
		Err(s
			.iter()
			.position(|b| !(0x20..=0x7e).contains(b) && !(0xa0..=0xff).contains(b))
			.unwrap())
	}
}

/// Format an ASCII-ish byte string
pub struct FmtAsciiIsh<'a>(pub &'a [u8]);

impl Debug for FmtAsciiIsh<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_fmt(format_args!("b\"{self}\""))
	}
}

impl Display for FmtAsciiIsh<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		use std::fmt::Write;

		const ASCII_CONTROL_ESCAPES: &[(u8, &str)] = &[
			(b'\0', "\\0"),
			(b'\t', "\\t"),
			(b'\n', "\\n"),
			(b'\r', "\\r"),
			(b'"', "\\\""),
			(b'\\', "\\\\"),
		];

		for byte in self.0.iter().copied() {
			if let Ok(escaped) = ASCII_CONTROL_ESCAPES.binary_search_by_key(&byte, |&(c, _)| c) {
				f.write_str(ASCII_CONTROL_ESCAPES[escaped].1)?;
			} else if byte.is_ascii_graphic() || byte == b' ' {
				f.write_char(byte as char)?;
			} else {
				f.write_fmt(format_args!("\\x{byte:02x}"))?;
			}
		}

		Ok(())
	}
}

/// Format a byte string as UTF-8 if possible, otherwise as an ASCII-ish string
pub struct FmtMaybeUtf8<'a>(pub &'a [u8]);

impl Debug for FmtMaybeUtf8<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if let Ok(s) = str::from_utf8(self.0) {
			write!(f, "\"{s}\"")
		} else {
			write!(f, "{:?}", FmtAsciiIsh(self.0))
		}
	}
}

impl Display for FmtMaybeUtf8<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if let Ok(s) = str::from_utf8(self.0) {
			write!(f, "{s}")
		} else {
			write!(f, "{}", FmtAsciiIsh(self.0))
		}
	}
}

/// Format a socket address, if it's known
pub struct FmtMaybeAddr<'a, E>(pub &'a Result<SocketAddr, E>);

impl<E: Debug> Debug for FmtMaybeAddr<'_, E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self.0 {
			Ok(addr) => write!(f, "{addr:?}"),
			Err(err) => write!(f, "{err:?}"),
		}
	}
}

impl<E> Display for FmtMaybeAddr<'_, E> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		if let Ok(addr) = self.0 {
			write!(f, "{addr}")
		} else {
			write!(f, "[address unknown]")
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{
		array,
		net::{IpAddr, Ipv4Addr, Ipv6Addr},
	};

	use super::*;

	#[test]
	fn decode_iso_8859_1() {
		assert_eq!(
			super::decode_iso_8859_1(b"Hello, World!"),
			Ok("Hello, World!".into())
		);
		assert_eq!(
			super::decode_iso_8859_1(b"\xa1Hello, World!"),
			Ok("¡Hello, World!".into())
		);
		assert_eq!(
			super::decode_iso_8859_1("Witaj świecie!".as_bytes()),
			Err(7) // 'ś' is [0xc5, 0x9b], where 0xc5 in ISO 8859/1 is 'Å', so the error is on 0x9b
		);
		assert_eq!(super::decode_iso_8859_1("🏳️‍🌈".as_bytes()), Err(1));
		assert_eq!(super::decode_iso_8859_1(&[][..]), Ok("".into()));
		assert_eq!(
			super::decode_iso_8859_1("äöü".as_bytes()),
			Ok("Ã¤Ã¶Ã¼".into())
		);
		assert_eq!(
			super::decode_iso_8859_1(&array::from_fn::<_, 256, _>(|i| i as u8)[..]),
			Err(0)
		);
		assert_eq!(
			super::decode_iso_8859_1(
				&array::from_fn::<_, { 256 - b' ' as usize }, _>(|i| i as u8 + b' ')[..]
			),
			Err((0x7f - b' ') as usize)
		);
		assert_eq!(
			super::decode_iso_8859_1(&(0x20..=0x7e).chain(0xa0..=0xff).collect::<Vec<_>>()),
			Ok(concat!(
				" !\"#$%&'()*+,-./",
				"0123456789:;<=>?",
				"@ABCDEFGHIJKLMNO",
				"PQRSTUVWXYZ[\\]^_",
				"`abcdefghijklmno",
				"pqrstuvwxyz{|}~",
				" ¡¢£¤¥¦§¨©ª«¬­®¯", // soft hyphen between ¬ and ®
				"°±²³´µ¶·¸¹º»¼½¾¿",
				"ÀÁÂÃÄÅÆÇÈÉÊËÌÍÎÏ",
				"ÐÑÒÓÔÕÖ×ØÙÚÛÜÝÞß",
				"àáâãäåæçèéêëìíîï",
				"ðñòóôõö÷øùúûüýþÿ"
			)
			.into())
		);
	}

	#[test]
	fn fmt_ascii_ish_display() {
		assert_eq!(format!("a {} c", FmtAsciiIsh(b"b")), r"a b c");
		assert_eq!(format!("a {} c", FmtAsciiIsh(b"123")), r"a 123 c");
		assert_eq!(format!("a {} c", FmtAsciiIsh(b"\0b")), r"a \0b c");
		assert_eq!(
			format!("a {} c", FmtAsciiIsh(&[0xff, 0xee][..])),
			r"a \xff\xee c"
		);
		assert_eq!(
			format!("a {} c", FmtAsciiIsh(b"\xaa \n \r \t \\ \x00 \0 \' ' \"")),
			r#"a \xaa \n \r \t \\ \0 \0 ' ' \" c"#
		);
		assert_eq!(
			format!("a {} c", FmtAsciiIsh("🏳️‍🌈".as_bytes())),
			r"a \xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xf0\x9f\x8c\x88 c"
		);
		assert_eq!(
			format!(
				"a {} c",
				FmtAsciiIsh("𝒷 \n \r \t \\ \x00 \0 \' ' \"".as_bytes())
			),
			r#"a \xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \" c"#
		);
	}

	#[test]
	fn fmt_ascii_ish_debug() {
		assert_eq!(format!("a {:?} c", FmtAsciiIsh(b"b")), r#"a b"b" c"#);
		assert_eq!(format!("a {:?} c", FmtAsciiIsh(b"123")), r#"a b"123" c"#);
		assert_eq!(format!("a {:?} c", FmtAsciiIsh(b"\0b")), r#"a b"\0b" c"#);
		assert_eq!(
			format!("a {:?} c", FmtAsciiIsh(&[0xff, 0xee][..])),
			r#"a b"\xff\xee" c"#
		);
		assert_eq!(
			format!("a {:?} c", FmtAsciiIsh(b"\xaa \n \r \t \\ \x00 \0 \' ' \"")),
			r#"a b"\xaa \n \r \t \\ \0 \0 ' ' \"" c"#
		);
		assert_eq!(
			format!("a {:?} c", FmtAsciiIsh("🏳️‍🌈".as_bytes())),
			r#"a b"\xf0\x9f\x8f\xb3\xef\xb8\x8f\xe2\x80\x8d\xf0\x9f\x8c\x88" c"#
		);
		assert_eq!(
			format!(
				"a {:?} c",
				FmtAsciiIsh("𝒷 \n \r \t \\ \x00 \0 \' ' \"".as_bytes())
			),
			r#"a b"\xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \"" c"#
		);
	}

	#[test]
	fn fmt_maybe_utf8_display() {
		assert_eq!(format!("a {} c", FmtMaybeUtf8(b"b")), "a b c");
		assert_eq!(format!("a {} c", FmtMaybeUtf8(b"123")), "a 123 c");
		assert_eq!(format!("a {} c", FmtMaybeUtf8(b"\0b")), "a \0b c");
		assert_eq!(
			format!("a {} c", FmtMaybeUtf8(&[0xff, 0xee][..])),
			r"a \xff\xee c"
		);
		assert_eq!(
			format!("a {} c", FmtMaybeUtf8(b"\xaa \n \r \t \\ \x00 \0 \' ' \"")),
			r#"a \xaa \n \r \t \\ \0 \0 ' ' \" c"#
		);
		assert_eq!(format!("a {} c", FmtMaybeUtf8("🏳️‍🌈".as_bytes())), "a 🏳️‍🌈 c");
		assert_eq!(
			format!(
				"a {} c",
				FmtMaybeUtf8("𝒷 \n \r \t \\ \x00 \0 \' ' \"".as_bytes())
			),
			"a 𝒷 \n \r \t \\ \0 \0 ' ' \" c"
		);
		assert_eq!(
			format!(
				"a {} c",
				FmtAsciiIsh(b"\xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \"")
			),
			r#"a \xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \" c"#
		);
	}

	#[test]
	fn fmt_maybe_utf8_debug() {
		assert_eq!(format!("a {:?} c", FmtMaybeUtf8(b"b")), "a \"b\" c");
		assert_eq!(format!("a {:?} c", FmtMaybeUtf8(b"123")), "a \"123\" c");
		assert_eq!(format!("a {:?} c", FmtMaybeUtf8(b"\0b")), "a \"\0b\" c");
		assert_eq!(
			format!("a {:?} c", FmtMaybeUtf8(&[0xff, 0xee][..])),
			r#"a b"\xff\xee" c"#
		);
		assert_eq!(
			format!(
				"a {:?} c",
				FmtMaybeUtf8(b"\xaa \n \r \t \\ \x00 \0 \' ' \"")
			),
			r#"a b"\xaa \n \r \t \\ \0 \0 ' ' \"" c"#
		);
		assert_eq!(
			format!("a {:?} c", FmtMaybeUtf8("🏳️‍🌈".as_bytes())),
			"a \"🏳️‍🌈\" c"
		);
		assert_eq!(
			format!(
				"a {:?} c",
				FmtMaybeUtf8("𝒷 \n \r \t \\ \x00 \0 \' ' \"".as_bytes())
			),
			"a \"𝒷 \n \r \t \\ \0 \0 ' ' \"\" c"
		);
		assert_eq!(
			format!(
				"a {:?} c",
				FmtAsciiIsh(b"\xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \"")
			),
			r#"a b"\xf0\x9d\x92\xb7 \n \r \t \\ \0 \0 ' ' \"" c"#
		);
	}

	#[test]
	fn fmt_maybe_addr_display() {
		assert_eq!(
			format!(
				"a {} c",
				FmtMaybeAddr(&Ok::<_, ()>(SocketAddr::new(
					IpAddr::V4(Ipv4Addr::LOCALHOST),
					80
				)))
			),
			r#"a 127.0.0.1:80 c"#
		);
		assert_eq!(
			format!(
				"a {} c",
				FmtMaybeAddr(&Ok::<_, ()>(SocketAddr::new(
					IpAddr::V6(Ipv6Addr::LOCALHOST),
					80
				)))
			),
			r#"a [::1]:80 c"#
		);
		assert_eq!(
			format!("a {} c", FmtMaybeAddr(&Err(()))),
			r#"a [address unknown] c"#
		);
		assert_eq!(
			format!("a {} c", FmtMaybeAddr(&Err("b"))),
			r#"a [address unknown] c"#
		);
	}

	#[test]
	fn fmt_maybe_addr_debug() {
		assert_eq!(
			format!(
				"a {:?} c",
				FmtMaybeAddr(&Ok::<_, ()>(SocketAddr::new(
					IpAddr::V4(Ipv4Addr::LOCALHOST),
					80
				)))
			),
			r#"a 127.0.0.1:80 c"#
		);
		assert_eq!(
			format!(
				"a {:?} c",
				FmtMaybeAddr(&Ok::<_, ()>(SocketAddr::new(
					IpAddr::V6(Ipv6Addr::LOCALHOST),
					80
				)))
			),
			r#"a [::1]:80 c"#
		);
		assert_eq!(format!("a {:?} c", FmtMaybeAddr(&Err(()))), r#"a () c"#);
		assert_eq!(format!("a {:?} c", FmtMaybeAddr(&Err("b"))), r#"a "b" c"#);
	}
}
