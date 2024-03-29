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

pub mod app_paths;
pub mod config;
pub mod template;
pub mod updater_signature;
pub mod web_dev_server;

use std::{
	collections::HashMap,
	path::{Path, PathBuf}
};

pub fn command_env(debug: bool) -> HashMap<&'static str, String> {
	let mut map = HashMap::new();
	map.insert("MILLENNIUM_PLATFORM_VERSION", os_info::get().version().to_string());

	if debug {
		map.insert("MILLENNIUM_DEBUG", "true".into());
	}

	map
}

pub fn resolve_millennium_path<P: AsRef<Path>>(path: P, crate_name: &str) -> PathBuf {
	let path = path.as_ref();
	if path.is_absolute() {
		path.join(crate_name)
	} else {
		PathBuf::from("..").join(path).join(crate_name)
	}
}
