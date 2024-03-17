use std::{
	env,
	error::Error,
	fs,
	path::{Path, PathBuf},
};

use ignore::gitignore::Gitignore;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UserInfo {
	username: String,
	full_name: String,
	info: String,
}

const QUOTE_VALID_CHARACTERS: &[u8] =
	r#"!#"$%&'()*+,-./0123456789:;<=>?ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz~ "#
		.as_bytes();
const QUOTE_MAX_LEN: usize = 510;

const USERNAME_VALID_CHARACTERS: &[u8] =
	r#"-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz"#.as_bytes();

fn main() -> Result<(), Box<dyn Error>> {
	println!("cargo:rerun-if-changed=build.rs");

	eprintln!("Adding file system entries");

	let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("fs.rs");
	fs::write(out_path, get_fs().as_bytes())?;

	eprintln!("Added file system entries");

	eprintln!("Adding user info");

	let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("users.rs");
	fs::write(out_path, get_users().as_bytes())?;

	eprintln!("Added user info");

	let mut quotes = get_quotes();
	quotes.sort_unstable();
	quotes.dedup();

	eprintln!("Adding {} quotes", quotes.len());

	let mut out = String::new();
	for quote in quotes {
		let quote = asciify(&quote);
		let quote = quote.trim();

		if quote.bytes().all(|b| QUOTE_VALID_CHARACTERS.contains(&b))
			&& quote.len() <= QUOTE_MAX_LEN
		{
			out += quote;
			out += "\n";
		} else {
			eprintln!(
				"Not adding quote because it contains invalid characters or is too long: {quote}"
			);
		}
	}

	let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("quotes.txt");
	fs::write(out_path, out.as_bytes())?;

	eprintln!("Added {} quotes", out.lines().count());

	let mut usernames = get_usernames();
	usernames.sort_unstable();
	usernames.dedup();

	eprintln!("Adding {} usernames", usernames.len());

	let mut out = String::new();
	for username in usernames {
		let username = username.to_lowercase().replace([' ', '\''], "_");
		let username = decancer::cure(&username);
		let username = username.trim();

		if username
			.bytes()
			.all(|b| USERNAME_VALID_CHARACTERS.contains(&b))
		{
			out += username;
			out += "\n";
		} else {
			eprintln!("Not adding username because it contains invalid bytes: {username}");
		}
	}

	let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("usernames.txt");
	fs::write(out_path, out.as_bytes())?;

	eprintln!("Added {} usernames", out.lines().count());

	Ok(())
}

fn asciify(s: &str) -> String {
	let mut res = String::with_capacity(s.len());

	for char in s.chars() {
		if char.is_ascii() {
			res.push(char);
		} else {
			let mut new = decancer::cure_char(char).to_string();
			if char.is_uppercase() {
				new.make_ascii_uppercase();
			}
			res.push_str(&new);
		}
	}

	res
}

/// Get file system entries as code
///
/// The returned string is a Rust literal in the format of `Entry::Directory {
/// name: "", entries: &[Entry::File { ... }] }`
fn get_fs() -> String {
	fn get_fs_entries(path: &Path, ignorer: &Gitignore, is_project_root_dir: bool) -> String {
		if fs::metadata(path)
			.expect("can't access file system metadata")
			.is_dir()
		{
			format!(
				r#"Entry::Directory {{ name: "{}", entries: &[{}] }}"#,
				if is_project_root_dir {
					PathBuf::new()
				} else {
					PathBuf::from(&path.file_name().expect("file path has no file name"))
				}
				.display(),
				fs::read_dir(path)
					.expect("can't read directory")
					.filter_map(|e| {
						let e = e.expect("can't read directory entry");

						(!ignorer
							.matched_path_or_any_parents(
								&e.path(),
								e.metadata()
									.expect("can't get directory entry metadata")
									.is_dir(),
							)
							.is_ignore())
						.then(|| get_fs_entries(&e.path(), ignorer, false))
					})
					.collect::<Vec<_>>()
					.join(", ")
			)
		} else {
			format!(
				r##"Entry::File {{ name: "{}", contents: include_str!(r#"{}"#) }}"##,
				PathBuf::from(&path.file_name().expect("file path has no file name")).display(),
				path.canonicalize()
					.expect("can't canonicalize file path")
					.display()
			)
		}
	}

	let (ignorer, err) = Gitignore::new(
		PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
			.join("./.gitignore")
			.canonicalize()
			.expect("can't canonicalize .gitignore path"),
	);

	if let Some(err) = err {
		eprintln!("Error instantiating .gitignore-based ignorer: {err}");
	}

	get_fs_entries(
		&PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
			.canonicalize()
			.expect("can't canonicalize project path"),
		&ignorer,
		true,
	)
}

/// Get user information as code
///
/// The returned string is a Rust literal in the format of `&[UserInfo {
/// username: "alice-original", full_name: "Alice Original", info: None }, ...]`
fn get_users() -> String {
	fn ensure_valid_username(username: String) -> Option<String> {
		let username = username.to_lowercase().replace([' ', '\''], "_");
		let username = decancer::cure(&username);
		let username = username.trim();

		if username
			.bytes()
			.all(|b| USERNAME_VALID_CHARACTERS.contains(&b))
		{
			Some(username.to_string())
		} else {
			eprintln!("Not adding username because it contains invalid bytes: {username}");
			None
		}
	}

	fn ensure_valid_full_name(full_name: String) -> Option<String> {
		let full_name = full_name.trim();

		if !full_name.is_ascii() {
			eprintln!(
				"WARNING: full name \"{full_name}\" contains non-ascii characters. This is \
				 probably fine, but might not always work."
			);
		}

		if !full_name.contains(['\r', '\n', '"']) {
			Some(full_name.to_string())
		} else {
			eprintln!("Not adding full name because it contains invalid characters: {full_name}");
			None
		}
	}

	fn ensure_valid_info(info: String) -> Option<String> {
		let info = info.lines().collect::<Vec<_>>().join("\r\n");
		let info = info.trim();

		if !info.is_ascii() {
			eprintln!(
				"WARNING: user info \"{info}\" contains non-ascii characters. This is probably \
				 fine, but might not always work."
			);
		}

		if !info.contains("\"###") {
			Some(if info.is_empty() {
				"None".to_string()
			} else {
				format!(r####"Some(r###"{info}"###)"####)
			})
		} else {
			eprintln!("Not adding user info because it contains invalid characters: {info}");
			None
		}
	}

	format!(
		r#"&[ {} ]"#,
		get_user_info()
			.into_iter()
			.filter_map(|ui| Some(format!(
				r#"UserInfo {{ username: "{}", full_name: "{}", info: {} }}"#,
				ensure_valid_username(ui.username)?,
				ensure_valid_full_name(ui.full_name)?,
				ensure_valid_info(ui.info)?
			)))
			.collect::<Vec<_>>()
			.join(", ")
	)
}

/// Get a vector of user names
///
/// The returned user names are directly from `data/users.json` and might
/// contain disallowed characters
fn get_usernames() -> Vec<String> {
	get_user_info().into_iter().map(|ui| ui.username).collect()
}

/// Get a vector of user info
///
/// The returned data is directly from `data/users.json` and might contain
/// disallowed characters
fn get_user_info() -> Vec<UserInfo> {
	println!("cargo:rerun-if-changed=data/users.json");

	serde_json::from_slice(
		&fs::read(
			PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("./data/users.json"),
		)
		.expect("can't read `users.json`"),
	)
	.expect("can't deserialize `users.json`")
}

/// Get a vector of quotes
///
/// The returned data is directly from `data/quotes.txt` and might contain
/// disallowed characters
fn get_quotes() -> Vec<String> {
	println!("cargo:rerun-if-changed=data/quotes.txt");

	fs::read_to_string(
		PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("./data/quotes.txt"),
	)
	.map(|s| s.lines().map(ToString::to_string).collect())
	.expect("`quotes.txt` could not be opened")
}
