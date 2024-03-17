//! A fake, read-only file system containing the source code of this project for
//! use with file-transferring protocols (e.g. FTP, HTTP, ...)

use std::{
	error::Error,
	fmt::{Display, Formatter, Result as FmtResult},
};

pub const PATH_VALID_CHARACTERS: &[u8] =
	b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-./_";

pub const FS: Entry<'static> = include!(concat!(env!("OUT_DIR"), "/fs.rs"));

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Entry<'e> {
	File {
		name: &'e str,
		contents: &'e str,
	},
	Directory {
		name: &'e str,
		entries: &'e [Entry<'e>],
	},
}

impl<'e> Entry<'e> {
	pub fn name(self) -> &'e str {
		match self {
			Self::Directory { name, .. } => name,
			Self::File { name, .. } => name,
		}
	}

	pub fn is_file(self) -> bool {
		matches!(self, Self::File { .. })
	}

	pub fn is_directory(self) -> bool {
		matches!(self, Self::Directory { .. })
	}
}

#[derive(Debug, Clone)]
pub enum FsError<'p> {
	NonAbsolutePath(&'p [u8]),
	InvalidPath(&'p [u8]),
	NotFound(&'p [u8]),
}

impl Display for FsError<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		match self {
			Self::NonAbsolutePath(path) => f.write_fmt(format_args!(
				"Path is not absolute: '{}'",
				String::from_utf8_lossy(path)
			)),
			Self::InvalidPath(path) => f.write_fmt(format_args!(
				"Invalid file name: '{}'",
				String::from_utf8_lossy(path)
			)),
			Self::NotFound(path) => f.write_fmt(format_args!(
				"File not found: '{}'",
				String::from_utf8_lossy(path)
			)),
		}
	}
}

impl Error for FsError<'_> {}

pub fn read(path: &[u8]) -> Result<Entry<'static>, FsError<'_>> {
	if path == b"/" {
		return Ok(FS);
	}

	if path.iter().next() != Some(&b'/') {
		return Err(FsError::NonAbsolutePath(path));
	}

	if !path.iter().all(|b| PATH_VALID_CHARACTERS.contains(b)) {
		return Err(FsError::InvalidPath(path));
	}

	let mut entry = &FS;
	for name in path.strip_suffix(b"/").unwrap_or(path)[1..].split(|&b| b == b'/') {
		match entry {
			Entry::Directory { entries, .. } => {
				entry = entries
					.iter()
					.find(|&e| e.name().as_bytes() == name)
					.ok_or(FsError::NotFound(path))?;
			}
			Entry::File { .. } => return Err(FsError::NotFound(path)),
		}
	}

	Ok(*entry)
}

pub fn root_entries() -> &'static [Entry<'static>] {
	let Entry::Directory { entries, .. } = FS else {
		unreachable!();
	};

	entries
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn read_this() {
		let file = read(b"/src/fs.rs").unwrap();
		assert!(file.is_file());
		assert_eq!(file.name(), "fs.rs");

		let Entry::File { contents, .. } = file else {
			unreachable!();
		};

		assert!(contents.contains("This is part of this file."));
	}

	#[test]
	fn read_build() {
		let file = read(b"/build.rs").unwrap();
		assert!(file.is_file());
		assert_eq!(file.name(), "build.rs");

		let Entry::File { contents, .. } = file else {
			unreachable!();
		};

		assert_eq!(contents, include_str!("../build.rs"));
	}

	#[test]
	fn read_no_target() {
		assert!(read(b"/target").is_err());
	}

	#[test]
	fn read_dir() {
		let dir = read(b"/src/services/").unwrap();
		assert!(dir.is_directory());
		assert_eq!(dir.name(), "services");

		let Ok(Entry::Directory { entries, .. }) = read(b"/src") else {
			panic!();
		};

		assert!(entries.iter().any(|e| e.name() == dir.name()));
	}

	#[test]
	fn read_nothing() {
		let entry = read(b"/src/foo/bar.rs");
		assert!(matches!(entry, Err(FsError::NotFound(_))));
	}

	#[test]
	fn read_nonabsolute() {
		let entry = read(b"./fs.rs");
		assert!(matches!(entry, Err(FsError::NonAbsolutePath(_))));
	}

	#[test]
	fn read_invalid() {
		let entry = read(b"/this is not a valid path in this context.txt");
		assert!(matches!(entry, Err(FsError::InvalidPath(_))));
	}

	#[test]
	fn root_has_root_entries() {
		assert_eq!(FS, Entry::Directory {
			name: "",
			entries: root_entries()
		});
	}
}
