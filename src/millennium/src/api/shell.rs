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

#![allow(clippy::tabs_in_doc_comments)]

//! Types and functions related to shell.

use std::str::FromStr;

use crate::ShellScope;

/// Program to use on the [`open()`] call.
pub enum Program {
	/// Use the `open` program.
	Open,
	/// Use the `start` program.
	Start,
	/// Use the `xdg-open` program.
	XdgOpen,
	/// Use the `gio` program.
	Gio,
	/// Use the `gnome-open` program.
	GnomeOpen,
	/// Use the `kde-open` program.
	KdeOpen,
	/// Use the `wslview` program.
	WslView,
	/// Use the `Firefox` program.
	Firefox,
	/// Use the `Google Chrome` program.
	Chrome,
	/// Use the `Chromium` program.
	Chromium,
	/// Use the `Safari` program.
	Safari
}

impl FromStr for Program {
	type Err = super::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let p = match s.to_lowercase().as_str() {
			"open" => Self::Open,
			"start" => Self::Start,
			"xdg-open" => Self::XdgOpen,
			"gio" => Self::Gio,
			"gnome-open" => Self::GnomeOpen,
			"kde-open" => Self::KdeOpen,
			"wslview" => Self::WslView,
			"firefox" => Self::Firefox,
			"chrome" | "google chrome" => Self::Chrome,
			"chromium" => Self::Chromium,
			"safari" => Self::Safari,
			_ => return Err(super::Error::UnknownProgramName(s.to_string()))
		};
		Ok(p)
	}
}

impl Program {
	pub(crate) fn name(self) -> &'static str {
		match self {
			Self::Open => "open",
			Self::Start => "start",
			Self::XdgOpen => "xdg-open",
			Self::Gio => "gio",
			Self::GnomeOpen => "gnome-open",
			Self::KdeOpen => "kde-open",
			Self::WslView => "wslview",

			#[cfg(target_os = "macos")]
			Self::Firefox => "Firefox",
			#[cfg(not(target_os = "macos"))]
			Self::Firefox => "firefox",

			#[cfg(target_os = "macos")]
			Self::Chrome => "Google Chrome",
			#[cfg(not(target_os = "macos"))]
			Self::Chrome => "google-chrome",

			#[cfg(target_os = "macos")]
			Self::Chromium => "Chromium",
			#[cfg(not(target_os = "macos"))]
			Self::Chromium => "chromium",

			#[cfg(target_os = "macos")]
			Self::Safari => "Safari",
			#[cfg(not(target_os = "macos"))]
			Self::Safari => "safari"
		}
	}
}

/// Opens path or URL with the program specified in `with`, or system default if
/// `None`.
///
/// The path will be matched against the shell open validation regex, defaulting to
/// `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`. A custom validation regex may be supplied in the config in `millennium
/// > allowlist > scope > open`.
///
/// # Examples
///
/// ```rust,no_run
/// use millennium::{api::shell::open, Manager};
/// millennium::Builder::default().setup(|app| {
/// 	// open the given URL on the system default browser
/// 	open(&app.shell_scope(), "https://example.com", None)?;
/// 	Ok(())
/// });
/// ```
pub fn open<P: AsRef<str>>(scope: &ShellScope, path: P, with: Option<Program>) -> crate::api::Result<()> {
	scope
		.open(path.as_ref(), with)
		.map_err(|err| crate::api::Error::Shell(format!("failed to open: {err}")))
}
