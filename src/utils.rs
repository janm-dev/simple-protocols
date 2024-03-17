//! Non-protocol-related utilities

use std::{
	borrow::Cow,
	fmt::{Debug, Display, Formatter, Result as FmtResult},
	net::SocketAddr,
	str,
};

/// Decode an ISO/IES 8859/1 string
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
			(b'\'', "\\'"),
		];

		for byte in self.0.iter().copied() {
			if byte.is_ascii_graphic() || byte == b' ' {
				f.write_char(byte as char)?;
			} else if let Ok(escaped) =
				ASCII_CONTROL_ESCAPES.binary_search_by_key(&byte, |&(c, _)| c)
			{
				f.write_str(ASCII_CONTROL_ESCAPES[escaped].1)?;
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
		f.write_fmt(format_args!("b\"{self}\""))
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
