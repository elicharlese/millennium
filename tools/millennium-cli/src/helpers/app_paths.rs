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
	cmp::Ordering,
	env::current_dir,
	ffi::OsStr,
	path::{Path, PathBuf}
};

use ignore::WalkBuilder;
use millennium_utils::config::parse::{folder_has_configuration_file, is_configuration_file, ConfigFormat};
use once_cell::sync::Lazy;

const MILLENNIUM_GITIGNORE: &[u8] = include_bytes!("../../millennium.gitignore");

fn lookup<F: Fn(&PathBuf) -> bool>(dir: &Path, checker: F) -> Option<PathBuf> {
	let mut default_gitignore = std::env::temp_dir();
	default_gitignore.push(".gitignore");
	if !default_gitignore.exists() {
		if let Ok(mut file) = std::fs::File::create(default_gitignore.clone()) {
			use std::io::Write;
			let _ = file.write_all(MILLENNIUM_GITIGNORE);
		}
	}

	let mut builder = WalkBuilder::new(dir);
	let _ = builder.add_ignore(default_gitignore);
	builder
		.add_custom_ignore_filename(".m1kignore")
		.add_custom_ignore_filename(".millenniumignore")
		.require_git(false)
		.ignore(false)
		.max_depth(Some(
			std::env::var("MILLENNIUM_PATH_DEPTH")
				.map(|d| {
					d.parse()
						.expect("`MILLENNIUM_PATH_DEPTH` environment variable must be a positive integer")
				})
				.unwrap_or(3)
		))
		.sort_by_file_path(|a, _| if a.extension().is_some() { Ordering::Less } else { Ordering::Greater });

	for entry in builder.build().flatten() {
		let path = dir.join(entry.path());
		if checker(&path) {
			return Some(path);
		}
	}
	None
}

fn get_millennium_dir() -> PathBuf {
	lookup(&current_dir().expect("failed to read cwd"), |path| folder_has_configuration_file(path) || is_configuration_file(path))
		.map(|p| if p.is_dir() { p } else { p.parent().unwrap().to_path_buf() })
		.unwrap_or_else(|| {
			panic!(
				"Couldn't recognize the current folder as a Millennium project. It must contain a `{}` or `{}` file in any subfolder.",
				ConfigFormat::Toml.into_file_name(),
				ConfigFormat::Json5.into_file_name()
			)
		})
}

fn get_app_dir() -> Option<PathBuf> {
	lookup(&current_dir().expect("failed to read cwd"), |path| {
		if let Some(file_name) = path.file_name() {
			file_name == OsStr::new("package.json")
		} else {
			false
		}
	})
	.map(|p| p.parent().unwrap().to_path_buf())
}

pub fn app_dir() -> &'static PathBuf {
	static APP_DIR: Lazy<PathBuf> = Lazy::new(|| get_app_dir().unwrap_or_else(get_millennium_dir));
	&APP_DIR
}

pub fn millennium_dir() -> PathBuf {
	get_millennium_dir()
}
