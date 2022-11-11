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

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use json_patch::merge;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

use crate::config::Config;

/// All file extensions that are supported.
pub const EXTENSIONS_SUPPORTED: &[&str] = &["json", "json5", "jsonc", "toml"];

/// List of supported configuration formats.
pub const SUPPORTED_FORMATS: &[ConfigFormat] = &[ConfigFormat::Json5, ConfigFormat::Toml];

/// Millennium configuration file format.
#[derive(Debug, Copy, Clone)]
pub enum ConfigFormat {
	/// Deprecated JSON5 (.millenniumrc) format
	Json5,
	/// TOML (Millennium.toml) format
	Toml
}

impl ConfigFormat {
	/// Maps the config format to its base file name.
	pub fn into_file_name(self) -> &'static str {
		match self {
			Self::Json5 => ".millenniumrc",
			Self::Toml => "Millennium.toml"
		}
	}

	fn into_platform_file_name(self) -> &'static str {
		match self {
			Self::Json5 => {
				if cfg!(target_os = "macos") {
					".macos.millenniumrc"
				} else if cfg!(windows) {
					".windows.millenniumrc"
				} else {
					".linux.millenniumrc"
				}
			}
			Self::Toml => {
				if cfg!(target_os = "macos") {
					"Millennium.macos.toml"
				} else if cfg!(windows) {
					"Millennium.windows.toml"
				} else {
					"Millennium.linux.toml"
				}
			}
		}
	}
}

/// Represents all the errors that can happen while reading the config.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
	/// Failed to parse the config file in TOML format.
	#[error("unable to parse Millennium config file at {path} because {error}")]
	FormatToml {
		/// The path that failed to parse into TOML.
		path: PathBuf,
		/// The parsing [`toml::Error`].
		error: ::toml::de::Error
	},
	/// Failed to parse the config file in JSON5 format.
	#[error("unable to parse Millennium config file at {path} because {error}")]
	FormatJson {
		/// The path that failed to parse into JSON5.
		path: PathBuf,
		/// The parsing [`json5::Error`].
		error: ::json5::Error
	},
	/// Unknown file extension encountered.
	#[error("unsupported format encountered {0}")]
	UnsupportedFormat(String),
	/// A generic IO error with context of what caused it.
	#[error("unable to read Millennium config file at {path} because {error}")]
	Io {
		/// The path the IO error occured on.
		path: PathBuf,
		/// The [`std::io::Error`].
		error: std::io::Error
	}
}

/// Reads the configuration from the given root directory.
///
/// It first looks for a `.millenniumrc` file on the given directory. The file
/// must exist. Then it looks for a platform-specific configuration file:
/// - `.millenniumrc.macos` on macOS
/// - `.millenniumrc.linux` on Linux
/// - `.millenniumrc.windows` on Windows
/// Merging the configurations using [JSON Merge Patch (RFC 7396)].
///
/// [JSON Merge Patch (RFC 7396)]: https://datatracker.ietf.org/doc/html/rfc7396.
pub fn read_from(root_dir: PathBuf) -> Result<Value, ConfigError> {
	let mut config: Value = parse_value(root_dir.join("Millennium.toml"))?.0;
	if let Some((platform_config, _)) = read_platform(root_dir)? {
		merge(&mut config, &platform_config);
	}
	Ok(config)
}

/// Reads the platform-specific configuration file in the given directory.
pub fn read_platform(root_dir: PathBuf) -> Result<Option<(Value, PathBuf)>, ConfigError> {
	let platform_config_path = root_dir.join(ConfigFormat::Toml.into_platform_file_name());
	if does_supported_file_name_exist(&platform_config_path) {
		let (platform_config, path): (Value, PathBuf) = parse_value(platform_config_path)?;
		Ok(Some((platform_config, path)))
	} else {
		Ok(None)
	}
}

/// Check if a supported config file exists at path.
///
/// The passed path is expected to be the path to the "default" configuration
/// format, in this case JSON with `.json`.
pub fn does_supported_file_name_exist(path: impl Into<PathBuf>) -> bool {
	let path = path.into();
	let source_file_name = path.file_name().unwrap().to_str().unwrap();
	let lookup_platform_config = SUPPORTED_FORMATS
		.iter()
		.any(|format| source_file_name == format.into_platform_file_name());
	SUPPORTED_FORMATS.iter().any(|format| {
		path.with_file_name(if lookup_platform_config { format.into_platform_file_name() } else { format.into_file_name() })
			.exists()
	})
}

/// Parse the config from path, including alternative formats.
///
/// Hierarchy:
/// 1. Check if `.millenniumrc` or `.millenniumrc.json` exists
///   a. Parse it with `serde_json`
///   b. Parse it with `json5` if `serde_json` fails
///   c. Return original `serde_json` error if all above steps failed
/// 2. Check if `.millenniumrc.json5` exists
///   a. Parse it with `json5`
///   b. Return error if all above steps failed
/// 3. Return error if all above steps failed
pub fn parse(path: impl Into<PathBuf>) -> Result<(Config, PathBuf), ConfigError> {
	do_parse(path.into())
}

/// See [`parse`] for specifics, returns a JSON [`Value`] instead of [`Config`].
pub fn parse_value(path: impl Into<PathBuf>) -> Result<(Value, PathBuf), ConfigError> {
	do_parse(path.into())
}

fn do_parse<D: DeserializeOwned>(path: PathBuf) -> Result<(D, PathBuf), ConfigError> {
	let file_name = path.file_name().map(OsStr::to_string_lossy).unwrap_or_default();
	let lookup_platform_config = SUPPORTED_FORMATS.iter().any(|format| file_name == format.into_platform_file_name());

	let toml = path.with_file_name(if lookup_platform_config {
		ConfigFormat::Toml.into_platform_file_name()
	} else {
		ConfigFormat::Toml.into_file_name()
	});
	let json5 = path.with_file_name(if lookup_platform_config {
		ConfigFormat::Json5.into_platform_file_name()
	} else {
		ConfigFormat::Json5.into_file_name()
	});

	let path_ext = path.extension().map(OsStr::to_string_lossy).unwrap_or_default();

	if path.exists() {
		let raw = read_to_string(&toml)?;
		do_parse_toml(&raw, &path).map(|config| (config, toml))
	} else if json5.exists() {
		let raw = read_to_string(&json5)?;
		do_parse_json(&raw, &path).map(|config| (config, json5))
	} else if !EXTENSIONS_SUPPORTED.contains(&path_ext.as_ref()) {
		Err(ConfigError::UnsupportedFormat(path_ext.to_string()))
	} else {
		Err(ConfigError::Io {
			path,
			error: std::io::ErrorKind::NotFound.into()
		})
	}
}

/// "Low-level" helper to parse JSON5 into a [`Config`].
///
/// `raw` should be the contents of the file that is represented by `path`.
pub fn parse_json(raw: &str, path: &Path) -> Result<Config, ConfigError> {
	do_parse_json(raw, path)
}

fn do_parse_json<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
	::json5::from_str(raw).map_err(|error| ConfigError::FormatJson { path: path.into(), error })
}

fn do_parse_toml<D: DeserializeOwned>(raw: &str, path: &Path) -> Result<D, ConfigError> {
	::toml::from_str(raw).map_err(|error| ConfigError::FormatToml { path: path.into(), error })
}

/// Helper function to wrap IO errors from [`std::fs::read_to_string`] into a
/// [`ConfigError`].
fn read_to_string(path: &Path) -> Result<String, ConfigError> {
	std::fs::read_to_string(path).map_err(|error| ConfigError::Io { path: path.into(), error })
}
