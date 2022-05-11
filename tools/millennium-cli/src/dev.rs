// Copyright 2022 pyke.io
//           2019-2021 Tauri Programme within The Commons Conservancy
//                     [https://tauri.studio/]
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
	env::set_current_dir,
	ffi::OsStr,
	fs::FileType,
	io::BufReader,
	path::{Path, PathBuf},
	process::{exit, Command},
	sync::{
		atomic::{AtomicBool, Ordering},
		mpsc::channel,
		Arc, Mutex
	},
	time::Duration
};

use anyhow::Context;
use clap::Parser;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use crate::{
	helpers::{
		app_paths::{app_dir, millennium_dir},
		command_env,
		config::{get as get_config, reload as reload_config, AppUrl, ConfigHandle, WindowUrl},
		manifest::{rewrite_manifest, Manifest},
		Logger
	},
	CommandExt, Result
};

static BEFORE_DEV: OnceCell<Mutex<Arc<SharedChild>>> = OnceCell::new();
static KILL_BEFORE_DEV_FLAG: OnceCell<AtomicBool> = OnceCell::new();

#[cfg(unix)]
const KILL_CHILDREN_SCRIPT: &[u8] = include_bytes!("../scripts/kill-children.sh");

const DEV_WATCHER_GITIGNORE: &[u8] = include_bytes!("../millennium-dev-watcher.gitignore");

#[derive(Debug, Parser)]
#[clap(about = "Start Millennium in development mode", trailing_var_arg(true))]
pub struct Options {
	/// Binary to use to run the application
	#[clap(short, long)]
	runner: Option<String>,
	/// Target triple to build against
	#[clap(short, long)]
	target: Option<String>,
	/// List of cargo features to activate
	#[clap(short, long)]
	features: Option<Vec<String>>,
	/// Exit on panic
	#[clap(short, long)]
	exit_on_panic: bool,
	/// JSON string or path to JSON file to merge with .millenniumrc
	#[clap(short, long)]
	config: Option<String>,
	/// Run the code in release mode
	#[clap(long = "release")]
	release_mode: bool,
	/// Command line arguments passed to the runner
	args: Vec<String>
}

pub fn command(options: Options) -> Result<()> {
	let r = command_internal(options);
	if r.is_err() {
		kill_before_dev_process();
	}
	r
}

fn command_internal(options: Options) -> Result<()> {
	let logger = Logger::new("millennium:dev");

	let millennium_path = millennium_dir();
	set_current_dir(&millennium_path).with_context(|| "failed to change current working directory")?;
	let merge_config = if let Some(config) = &options.config {
		Some(if config.starts_with('{') { config.to_string() } else { std::fs::read_to_string(&config)? })
	} else {
		None
	};
	let config = get_config(merge_config.as_deref())?;

	if let Some(before_dev) = &config.lock().unwrap().as_ref().unwrap().build.before_dev_command {
		if !before_dev.is_empty() {
			logger.log(format!("Running `{}`", before_dev));
			#[cfg(target_os = "windows")]
			let mut command = {
				let mut command = Command::new("cmd");
				command
					.arg("/S")
					.arg("/C")
					.arg(before_dev)
					.current_dir(app_dir())
					.envs(command_env(true))
					.pipe()?; // development build always includes debug information
				command
			};
			#[cfg(not(target_os = "windows"))]
			let mut command = {
				let mut command = Command::new("sh");
				command.arg("-c").arg(before_dev).current_dir(app_dir()).envs(command_env(true)).pipe()?; // development build always includes debug information
				command
			};

			let child = SharedChild::spawn(&mut command).unwrap_or_else(|_| panic!("failed to run `{}`", before_dev));
			let child = Arc::new(child);
			let child_ = child.clone();
			let logger_ = logger.clone();
			std::thread::spawn(move || {
				let status = child_.wait().expect("failed to wait on \"beforeDevCommand\"");
				if !(status.success() || KILL_BEFORE_DEV_FLAG.get().unwrap().load(Ordering::Relaxed)) {
					logger_.error("The \"beforeDevCommand\" terminated with a non-zero status code.");
					exit(status.code().unwrap_or(1));
				}
			});

			BEFORE_DEV.set(Mutex::new(child)).unwrap();
			KILL_BEFORE_DEV_FLAG.set(AtomicBool::default()).unwrap();

			let _ = ctrlc::set_handler(move || {
				kill_before_dev_process();
				exit(130);
			});
		}
	}

	let runner_from_config = config.lock().unwrap().as_ref().unwrap().build.runner.clone();
	let runner = options.runner.clone().or(runner_from_config).unwrap_or_else(|| "cargo".to_string());

	let manifest = {
		let (tx, rx) = channel();
		let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
		watcher.watch(millennium_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
		let manifest = rewrite_manifest(config.clone())?;
		loop {
			if let Ok(DebouncedEvent::NoticeWrite(_)) = rx.recv() {
				break;
			}
		}
		manifest
	};

	let mut cargo_features = config.lock().unwrap().as_ref().unwrap().build.features.clone().unwrap_or_default();
	if let Some(features) = &options.features {
		cargo_features.extend(features.clone());
	}

	let manually_killed_app = Arc::new(AtomicBool::default());

	if std::env::var_os("MILLENNIUM_SKIP_DEVSERVER_CHECK") != Some("true".into()) {
		if let AppUrl::Url(WindowUrl::External(dev_server_url)) = config.lock().unwrap().as_ref().unwrap().build.dev_path.clone() {
			let host = dev_server_url.host().unwrap_or_else(|| panic!("No host name in the URL"));
			let port = dev_server_url
				.port_or_known_default()
				.unwrap_or_else(|| panic!("No port number in the URL"));
			let addrs;
			let addr;
			let addrs = match host {
				url::Host::Domain(domain) => {
					use std::net::ToSocketAddrs;
					addrs = (domain, port).to_socket_addrs()?;
					addrs.as_slice()
				}
				url::Host::Ipv4(ip) => {
					addr = (ip, port).into();
					std::slice::from_ref(&addr)
				}
				url::Host::Ipv6(ip) => {
					addr = (ip, port).into();
					std::slice::from_ref(&addr)
				}
			};
			let mut i = 0;
			let sleep_interval = std::time::Duration::from_secs(2);
			let max_attempts = 90;
			loop {
				if std::net::TcpStream::connect(addrs).is_ok() {
					break;
				}
				if i % 3 == 0 {
					logger.warn(format!("Waiting for your frontend dev server to start on {}...", dev_server_url));
				}
				i += 1;
				if i == max_attempts {
					logger.error(format!(
						"Could not connect to `{}` after {}s. Please make sure that is the URL to your dev server.",
						dev_server_url,
						i * sleep_interval.as_secs()
					));
					exit(1);
				}
				std::thread::sleep(sleep_interval);
			}
		}
	}

	let process = start_app(&options, &runner, &manifest, &cargo_features, manually_killed_app.clone())?;
	let shared_process = Arc::new(Mutex::new(process));
	if let Err(e) = watch(shared_process.clone(), manually_killed_app, millennium_path, merge_config, config, options, runner, manifest, cargo_features) {
		shared_process.lock().unwrap().kill().with_context(|| "failed to kill app process")?;
		Err(e)
	} else {
		Ok(())
	}
}

fn lookup<F: FnMut(FileType, PathBuf)>(dir: &Path, mut f: F) {
	let mut default_gitignore = std::env::temp_dir();
	default_gitignore.push(".millennium-dev");
	let _ = std::fs::create_dir_all(&default_gitignore);
	default_gitignore.push(".gitignore");
	if !default_gitignore.exists() {
		if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
			use std::io::Write;
			let _ = file.write_all(DEV_WATCHER_GITIGNORE);
		}
	}

	let mut builder = ignore::WalkBuilder::new(dir);
	let _ = builder.add_ignore(default_gitignore);
	builder.require_git(false).ignore(false).max_depth(Some(1));

	for entry in builder.build().flatten() {
		f(entry.file_type().unwrap(), dir.join(entry.path()));
	}
}

#[allow(clippy::too_many_arguments)]
fn watch(
	process: Arc<Mutex<Arc<SharedChild>>>,
	manually_killed_app: Arc<AtomicBool>,
	millennium_path: PathBuf,
	merge_config: Option<String>,
	config: ConfigHandle,
	options: Options,
	runner: String,
	mut manifest: Manifest,
	cargo_features: Vec<String>
) -> Result<()> {
	let (tx, rx) = channel();

	let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
	lookup(&millennium_path, |file_type, path| {
		if path != millennium_path {
			let _ = watcher.watch(path, if file_type.is_dir() { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive });
		}
	});

	loop {
		if let Ok(event) = rx.recv() {
			let event_path = match event {
				DebouncedEvent::Create(path) => Some(path),
				DebouncedEvent::Remove(path) => Some(path),
				DebouncedEvent::Rename(_, dest) => Some(dest),
				DebouncedEvent::Write(path) => Some(path),
				_ => None
			};

			if let Some(event_path) = event_path {
				if event_path.file_name() == Some(OsStr::new(".millenniumrc")) {
					reload_config(merge_config.as_deref())?;
					manifest = rewrite_manifest(config.clone())?;
				} else {
					// When .millenniumrc is changed, rewrite_manifest will be called, which will trigger the watcher again.
					// The app should only be started when a file other than .millenniumrc is changed.
					manually_killed_app.store(true, Ordering::Relaxed);
					let mut p = process.lock().unwrap();
					p.kill().with_context(|| "failed to kill app process")?;
					// wait for the process to exit
					loop {
						if let Ok(Some(_)) = p.try_wait() {
							break;
						}
					}
					*p = start_app(&options, &runner, &manifest, &cargo_features, manually_killed_app.clone())?;
				}
			}
		}
	}
}

fn kill_before_dev_process() {
	if let Some(child) = BEFORE_DEV.get() {
		let child = child.lock().unwrap();
		KILL_BEFORE_DEV_FLAG.get().unwrap().store(true, Ordering::Relaxed);
		#[cfg(windows)]
		let _ = Command::new("powershell")
			.arg("-NoProfile")
			.arg("-Command")
			.arg(format!("function Kill-Tree {{ Param([int]$ppid); Get-CimInstance Win32_Process | Where-Object {{ $_.ParentProcessId -eq $ppid }} | ForEach-Object {{ Kill-Tree $_.ProcessId }}; Stop-Process -Id $ppid -ErrorAction SilentlyContinue }}; Kill-Tree {}", child.id()))
			.status();
		#[cfg(unix)]
		{
			let mut kill_children_script_path = std::env::temp_dir();
			kill_children_script_path.push("kill-children.sh");

			if !kill_children_script_path.exists() {
				if let Ok(mut file) = std::fs::File::create(&kill_children_script_path) {
					use std::{io::Write, os::unix::fs::PermissionsExt};
					let _ = file.write_all(KILL_CHILDREN_SCRIPT);
					let mut permissions = file.metadata().unwrap().permissions();
					permissions.set_mode(0o770);
					let _ = file.set_permissions(permissions);
				}
			}
			let _ = Command::new(&kill_children_script_path).arg(child.id().to_string()).output();
		}
		let _ = child.kill();
	}
}

fn start_app(options: &Options, runner: &str, manifest: &Manifest, features: &[String], manually_killed_app: Arc<AtomicBool>) -> Result<Arc<SharedChild>> {
	let mut command = Command::new(runner);
	command
		.env("CARGO_TERM_PROGRESS_WIDTH", terminal_size::terminal_size().map(|(w, _)| w.0).unwrap_or(80).to_string())
		.env("CARGO_TERM_PROGRESS_WHEN", "always");
	command.arg("run").arg("--color").arg("always");

	if !options.args.contains(&"--no-default-features".into()) {
		let manifest_features = manifest.features();
		let enable_features: Vec<String> = manifest_features
			.get("default")
			.cloned()
			.unwrap_or_default()
			.into_iter()
			.filter(|feature| {
				if let Some(manifest_feature) = manifest_features.get(feature) {
					!manifest_feature.contains(&"millennium/custom-protocol".into())
				} else {
					feature != "millennium/custom-protocol"
				}
			})
			.collect();
		command.arg("--no-default-features");
		if !enable_features.is_empty() {
			command.args(&["--features", &enable_features.join(",")]);
		}
	}

	if options.release_mode {
		command.args(&["--release"]);
	}

	if let Some(target) = &options.target {
		command.args(&["--target", target]);
	}

	if !features.is_empty() {
		command.args(&["--features", &features.join(",")]);
	}

	if !options.args.is_empty() {
		command.args(&options.args);
	}

	command.stdout(os_pipe::dup_stdout().unwrap());
	command.stderr(std::process::Stdio::piped());

	let child = SharedChild::spawn(&mut command).with_context(|| format!("failed to run {}", runner))?;
	let child_arc = Arc::new(child);
	let child_stderr = child_arc.take_stderr().unwrap();
	let mut stderr = BufReader::new(child_stderr);
	let stderr_lines = Arc::new(Mutex::new(Vec::new()));
	let stderr_lines_ = stderr_lines.clone();
	std::thread::spawn(move || {
		let mut buf = Vec::new();
		let mut lines = stderr_lines_.lock().unwrap();
		loop {
			buf.clear();
			match millennium_utils::io::read_line(&mut stderr, &mut buf) {
				Ok(s) if s == 0 => break,
				_ => ()
			}
			let line = String::from_utf8_lossy(&buf).into_owned();
			if line.ends_with('\r') {
				eprint!("{}", line);
			} else {
				eprintln!("{}", line);
			}
			lines.push(line);
		}
	});

	let child_clone = child_arc.clone();
	let exit_on_panic = options.exit_on_panic;
	std::thread::spawn(move || {
		let status = child_clone.wait().expect("failed to wait on child");
		if exit_on_panic {
			if !manually_killed_app.load(Ordering::Relaxed) {
				kill_before_dev_process();
				exit(status.code().unwrap_or(0));
			}
		} else {
			let is_cargo_compile_error = stderr_lines
				.lock()
				.unwrap()
				.last()
				.map(|l| l.contains("could not compile"))
				.unwrap_or_default();
			stderr_lines.lock().unwrap().clear();

			// if we're exiting on panic, we only exit if:
			// - the status is a success code (app closed)
			// - status code is a Cargo error & error is not a cargo compilation error
			if status.success() || (status.code() == Some(101) && !is_cargo_compile_error) {
				kill_before_dev_process();
				exit(status.code().unwrap_or(1));
			}
		}
	});

	Ok(child_arc)
}
