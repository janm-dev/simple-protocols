use std::{
	env,
	io::{Read, Write},
	net::{Ipv4Addr, SocketAddr, TcpStream},
	ops::{Deref, DerefMut},
	process::{Child, Command, Stdio},
	thread,
	time::Duration,
};

#[derive(Debug)]
struct KillOnDrop(Option<Child>);

impl KillOnDrop {
	fn new(child: Child) -> Self {
		Self(Some(child))
	}

	fn into_child(mut self) -> Child {
		self.0.take().unwrap()
	}
}

impl Deref for KillOnDrop {
	type Target = Child;

	fn deref(&self) -> &Self::Target {
		self.0.as_ref().unwrap()
	}
}

impl DerefMut for KillOnDrop {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.as_mut().unwrap()
	}
}

impl Drop for KillOnDrop {
	fn drop(&mut self) {
		if let Some(mut child) = self.0.take() {
			let id = child.id();
			eprintln!("Killing child process {id}");

			child.kill().unwrap();
			let out = child.wait_with_output().unwrap();

			eprintln!("{id} STDOUT:\n{}\n", String::from_utf8_lossy(&out.stdout));
			eprintln!("{id} STDERR:\n{}", String::from_utf8_lossy(&out.stderr));
		}
	}
}

#[test]
fn base_port() {
	let _server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stderr(Stdio::piped())
		.stdout(Stdio::piped())
		.args(["--log", "debug"])
		.args(["--base-port", "1234"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	// "A server listens for TCP connections on TCP port 7 [+ 1234 = 1241]."
	let mut tcp = TcpStream::connect_timeout(
		&SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 1241),
		Duration::from_secs(1),
	)
	.unwrap();

	tcp.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
	let mut buf = vec![0; 1024];

	// "Once a connection is established any data received" ...
	write!(tcp, "Hello, World!").unwrap();

	// ... "is sent back."
	let n = tcp.read(&mut buf).unwrap();
	assert!(&buf[..n] == b"Hello, World!");
}

#[test]
#[cfg(unix)]
fn ctrl_c_exit_unix() {
	use nix::{
		sys::signal::{self, Signal},
		unistd::Pid,
	};

	let server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stderr(Stdio::piped())
		.stdout(Stdio::piped())
		.args(["--log", "debug"])
		.args(["--base-port", "11000"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	signal::kill(Pid::from_raw(server.id() as _), Some(Signal::SIGINT)).unwrap();

	thread::sleep(Duration::from_secs(1));

	let output = server.into_child().wait_with_output().unwrap();
	let stderr = String::from_utf8_lossy(&output.stderr);

	assert!(stderr.contains("Simple Protocols Exiting"));
}

#[test]
fn non_ctrl_c_exit() {
	let mut server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stderr(Stdio::piped())
		.stdout(Stdio::piped())
		.args(["--log", "debug"])
		.args(["--base-port", "10000"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(1));

	server.kill().unwrap();

	thread::sleep(Duration::from_secs(1));

	let output = server.into_child().wait_with_output().unwrap();
	let stderr = String::from_utf8_lossy(&output.stderr);

	assert!(!stderr.contains("Simple Protocols Exiting"));
}

#[test]
fn env_overrides_arg() {
	let mut server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.envs([("SIMPLE_PROTOCOLS_LOG", "warn")])
		.args(["--log", "info"])
		.args(["--base-port", "12000"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(5));
	server.kill().unwrap();
	thread::sleep(Duration::from_secs(1));

	let output = server.into_child().wait_with_output().unwrap();
	let stderr = String::from_utf8_lossy(&output.stderr);

	dbg!(&stderr);

	assert!(stderr.contains("ERROR"));
	assert!(!stderr.contains("INFO"));
}

#[test]
fn arg_only() {
	let mut server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.args(["--log", "debug"])
		.args(["--base-port", "13000"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(5));
	server.kill().unwrap();
	thread::sleep(Duration::from_secs(1));

	let output = server.into_child().wait_with_output().unwrap();
	let stderr = String::from_utf8_lossy(&output.stderr);

	dbg!(&stderr);

	assert!(stderr.contains("INFO"));
	assert!(!stderr.contains("TRACE"));
}

#[test]
fn env_only() {
	let mut server = Command::new("./target/debug/simple-protocols")
		.env_clear()
		.envs(env::var_os("SystemRoot").map(|val| ("SystemRoot", val)))
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.envs([("SIMPLE_PROTOCOLS_LOG", "debug")])
		.args(["--base-port", "14000"])
		.spawn()
		.map(KillOnDrop::new)
		.unwrap();

	thread::sleep(Duration::from_secs(5));
	server.kill().unwrap();
	thread::sleep(Duration::from_secs(1));

	let output = server.into_child().wait_with_output().unwrap();
	let stderr = String::from_utf8_lossy(&output.stderr);

	dbg!(&stderr);

	assert!(stderr.contains("INFO"));
	assert!(!stderr.contains("TRACE"));
}
