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
	ffi::OsStr,
	fs::{self, File},
	io::{self, BufReader, BufWriter},
	path::Path,
	process::{Command, Output, Stdio},
	sync::{Arc, Mutex}
};

use log::debug;

/// Returns true if the path has a filename indicating that it is a high-desity
/// "retina" icon.  Specifically, returns true the the file stem ends with
/// "@2x" (a convention specified by the [Apple developer docs](
/// https://developer.apple.com/library/mac/documentation/GraphicsAnimation/Conceptual/HighResolutionOSX/Optimizing/Optimizing.html)).
#[allow(dead_code)]
pub fn is_retina<P: AsRef<Path>>(path: P) -> bool {
	path.as_ref()
		.file_stem()
		.and_then(OsStr::to_str)
		.map(|stem| stem.ends_with("@2x"))
		.unwrap_or(false)
}

/// Creates a new file at the given path, creating any parent directories as
/// needed.
pub fn create_file(path: &Path) -> crate::Result<BufWriter<File>> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(parent)?;
	}
	let file = File::create(path)?;
	Ok(BufWriter::new(file))
}

/// Makes a symbolic link to a directory.
#[cfg(unix)]
#[allow(dead_code)]
fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
	std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a directory.
#[cfg(windows)]
fn symlink_dir(src: &Path, dst: &Path) -> io::Result<()> {
	std::os::windows::fs::symlink_dir(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(unix)]
#[allow(dead_code)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
	std::os::unix::fs::symlink(src, dst)
}

/// Makes a symbolic link to a file.
#[cfg(windows)]
fn symlink_file(src: &Path, dst: &Path) -> io::Result<()> {
	std::os::windows::fs::symlink_file(src, dst)
}

/// Copies a regular file from one path to another, creating any parent
/// directories of the destination path as necessary.  Fails if the source path
/// is a directory or doesn't exist.
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<()> {
	let from = from.as_ref();
	let to = to.as_ref();
	if !from.exists() {
		return Err(crate::Error::GenericError(format!("{from:?} does not exist")));
	}
	if !from.is_file() {
		return Err(crate::Error::GenericError(format!("{from:?} is not a file")));
	}
	let dest_dir = to.parent().expect("No data in parent");
	fs::create_dir_all(dest_dir)?;
	fs::copy(from, to)?;
	Ok(())
}

/// Recursively copies a directory file from one path to another, creating any
/// parent directories of the destination path as necessary.  Fails if the
/// source path is not a directory or doesn't exist, or if the destination path
/// already exists.
#[allow(dead_code)]
pub fn copy_dir(from: &Path, to: &Path) -> crate::Result<()> {
	if !from.exists() {
		return Err(crate::Error::GenericError(format!("{from:?} does not exist")));
	}
	if !from.is_dir() {
		return Err(crate::Error::GenericError(format!("{from:?} is not a Directory")));
	}
	if to.exists() {
		return Err(crate::Error::GenericError(format!("{from:?} already exists")));
	}
	let parent = to.parent().expect("No data in parent");
	fs::create_dir_all(parent)?;
	for entry in walkdir::WalkDir::new(from) {
		let entry = entry?;
		debug_assert!(entry.path().starts_with(from));
		let rel_path = entry.path().strip_prefix(from)?;
		let dest_path = to.join(rel_path);
		if entry.file_type().is_symlink() {
			let target = fs::read_link(entry.path())?;
			if entry.path().is_dir() {
				symlink_dir(&target, &dest_path)?;
			} else {
				symlink_file(&target, &dest_path)?;
			}
		} else if entry.file_type().is_dir() {
			fs::create_dir(dest_path)?;
		} else {
			fs::copy(entry.path(), dest_path)?;
		}
	}
	Ok(())
}

pub trait CommandExt {
	fn output_ok(&mut self) -> crate::Result<Output>;
}

impl CommandExt for Command {
	fn output_ok(&mut self) -> crate::Result<Output> {
		let program = self.get_program().to_string_lossy().into_owned();
		debug!(action = "Running"; "Command `{} {}`", program, self.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));

		self.stdout(Stdio::piped());
		self.stderr(Stdio::piped());

		let mut child = self.spawn()?;

		let mut stdout = child.stdout.take().map(BufReader::new).unwrap();
		let stdout_lines = Arc::new(Mutex::new(Vec::new()));
		let stdout_lines_ = stdout_lines.clone();
		std::thread::spawn(move || {
			let mut buf = Vec::new();
			let mut lines = stdout_lines_.lock().unwrap();
			loop {
				buf.clear();
				match millennium_utils::io::read_line(&mut stdout, &mut buf) {
					Ok(s) if s == 0 => break,
					_ => {}
				}
				debug!(action = "Stdout"; "{}", String::from_utf8_lossy(&buf));
				lines.extend(buf.clone());
				lines.push(b'\n');
			}
		});

		let mut stderr = child.stderr.take().map(BufReader::new).unwrap();
		let stderr_lines = Arc::new(Mutex::new(Vec::new()));
		let stderr_lines_ = stderr_lines.clone();
		std::thread::spawn(move || {
			let mut buf = Vec::new();
			let mut lines = stderr_lines_.lock().unwrap();
			loop {
				buf.clear();
				match millennium_utils::io::read_line(&mut stderr, &mut buf) {
					Ok(s) if s == 0 => break,
					_ => {}
				}
				debug!(action = "Stderr"; "{}", String::from_utf8_lossy(&buf));
				lines.extend(buf.clone());
				lines.push(b'\n');
			}
		});

		let status = child.wait()?;
		let output = Output {
			status,
			stdout: std::mem::take(&mut *stdout_lines.lock().unwrap()),
			stderr: std::mem::take(&mut *stderr_lines.lock().unwrap())
		};

		if output.status.success() {
			Ok(output)
		} else {
			Err(crate::Error::GenericError(format!("failed to run {program}")))
		}
	}
}

#[cfg(test)]
mod tests {
	use std::{io::Write, path::PathBuf};

	use millennium_utils::resources::resource_relpath;

	use super::{create_file, is_retina};

	#[test]
	fn create_file_with_parent_dirs() {
		let tmp = tempfile::tempdir().expect("Unable to create temp dir");
		assert!(!tmp.path().join("parent").exists());
		{
			let mut file = create_file(&tmp.path().join("parent/file.txt")).expect("Failed to create file");
			writeln!(file, "Hello, world!").expect("unable to write file");
		}
		assert!(tmp.path().join("parent").is_dir());
		assert!(tmp.path().join("parent/file.txt").is_file());
	}

	#[cfg(not(windows))]
	#[test]
	fn copy_dir_with_symlinks() {
		// Create a directory structure that looks like this:
		//   ${TMP}/orig/
		//       sub/
		//           file.txt
		//       link -> sub/file.txt
		let tmp = tempfile::tempdir().expect("unable to create tempdir");
		{
			let mut file = create_file(&tmp.path().join("orig/sub/file.txt")).expect("Unable to create file");
			writeln!(file, "Hello, world!").expect("Unable to write to file");
		}
		super::symlink_file(&PathBuf::from("sub/file.txt"), &tmp.path().join("orig/link")).expect("Failed to create symlink");
		assert_eq!(std::fs::read(tmp.path().join("orig/link")).expect("Failed to read file").as_slice(), b"Hello, world!\n");
		// Copy ${TMP}/orig to ${TMP}/parent/copy, and make sure that the
		// directory structure, file, and symlink got copied correctly.
		super::copy_dir(&tmp.path().join("orig"), &tmp.path().join("parent/copy")).expect("Failed to copy dir");
		assert!(tmp.path().join("parent/copy").is_dir());
		assert!(tmp.path().join("parent/copy/sub").is_dir());
		assert!(tmp.path().join("parent/copy/sub/file.txt").is_file());
		assert_eq!(
			std::fs::read(tmp.path().join("parent/copy/sub/file.txt"))
				.expect("Failed to read file")
				.as_slice(),
			b"Hello, world!\n"
		);
		assert!(tmp.path().join("parent/copy/link").exists());
		assert_eq!(std::fs::read_link(tmp.path().join("parent/copy/link")).expect("Failed to read from symlink"), PathBuf::from("sub/file.txt"));
		assert_eq!(
			std::fs::read(tmp.path().join("parent/copy/link"))
				.expect("Failed to read from file")
				.as_slice(),
			b"Hello, world!\n"
		);
	}

	#[test]
	fn retina_icon_paths() {
		assert!(!is_retina("data/icons/512x512.png"));
		assert!(is_retina("data/icons/512x512@2x.png"));
	}

	#[test]
	fn resource_relative_paths() {
		assert_eq!(resource_relpath(&PathBuf::from("./data/images/button.png")), PathBuf::from("data/images/button.png"));
		assert_eq!(resource_relpath(&PathBuf::from("../../images/wheel.png")), PathBuf::from("_up_/_up_/images/wheel.png"));
		assert_eq!(resource_relpath(&PathBuf::from("/home/ferris/crab.png")), PathBuf::from("_root_/home/ferris/crab.png"));
	}
}
