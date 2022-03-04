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
	fmt,
	path::{Path, PathBuf},
	sync::{Arc, Mutex}
};

use glob::Pattern;
use millennium_utils::{
	config::{Config, FsAllowlistScope},
	Env, PackageInfo
};

use crate::api::path::parse as parse_path;

/// Scope for filesystem access.
#[derive(Clone)]
pub struct Scope {
	allow_patterns: Arc<Mutex<Vec<Pattern>>>
}

impl fmt::Debug for Scope {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Scope")
			.field(
				"allow_patterns",
				&self.allow_patterns.lock().unwrap().iter().map(|p| p.as_str()).collect::<Vec<&str>>()
			)
			.finish()
	}
}

fn push_pattern<P: AsRef<Path>>(list: &mut Vec<Pattern>, pattern: P) {
	let pattern: PathBuf = pattern.as_ref().components().collect();
	list.push(Pattern::new(&pattern.to_string_lossy()).expect("invalid glob pattern"));
	#[cfg(windows)]
	{
		list.push(Pattern::new(&format!("\\\\?\\{}", pattern.display())).expect("invalid glob pattern"));
	}
}

impl Scope {
	/// Creates a new scope from a `FsAllowlistScope` configuration.
	pub fn for_fs_api(config: &Config, package_info: &PackageInfo, env: &Env, scope: &FsAllowlistScope) -> Self {
		let mut allow_patterns = Vec::new();
		for path in &scope.0 {
			if let Ok(path) = parse_path(config, package_info, env, path) {
				push_pattern(&mut allow_patterns, path);
			}
		}
		Self {
			allow_patterns: Arc::new(Mutex::new(allow_patterns))
		}
	}

	/// Extend the allowed patterns with the given directory.
	///
	/// After this function has been called, the frontend will be able to use
	/// the Millennium API to read the directory and all of its files and
	/// subdirectories.
	pub fn allow_directory<P: AsRef<Path>>(&self, path: P, recursive: bool) {
		let path = path.as_ref().to_path_buf();
		let mut list = self.allow_patterns.lock().unwrap();

		// allow the directory to be read
		push_pattern(&mut list, &path);
		// allow its files and subdirectories to be read
		push_pattern(&mut list, path.join(if recursive { "**" } else { "*" }));
	}

	/// Extend the allowed patterns with the given file path.
	///
	/// After this function has been called, the frontend will be able to use
	/// the Millennium API to read the contents of this file.
	pub fn allow_file<P: AsRef<Path>>(&self, path: P) {
		push_pattern(&mut self.allow_patterns.lock().unwrap(), path);
	}

	/// Determines if the given path is allowed on this scope.
	pub fn is_allowed<P: AsRef<Path>>(&self, path: P) -> bool {
		let path = path.as_ref();
		let path = if !path.exists() {
			crate::Result::Ok(path.to_path_buf())
		} else {
			std::fs::canonicalize(path).map_err(Into::into)
		};

		if let Ok(path) = path {
			let path: PathBuf = path.components().collect();
			let allowed = self.allow_patterns.lock().unwrap().iter().any(|p| p.matches_path(&path));
			allowed
		} else {
			false
		}
	}
}
