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

use gdk::Atom;
use gtk::{TargetEntry, TargetFlags};

#[derive(Debug, Clone, Default)]
pub struct Clipboard;

const CLIPBOARD_TARGETS: [&str; 5] = ["UTF8_STRING", "TEXT", "STRING", "text/plain;charset=utf-8", "text/plain"];

impl Clipboard {
	pub(crate) fn write_text(&mut self, string: impl AsRef<str>) {
		let string = string.as_ref().to_string();

		let display = gdk::Display::default().unwrap();
		let clipboard = gtk::Clipboard::default(&display).unwrap();

		let targets: Vec<TargetEntry> = CLIPBOARD_TARGETS
			.iter()
			.enumerate()
			.map(|(i, target)| TargetEntry::new(target, TargetFlags::all(), i as u32))
			.collect();

		clipboard.set_with_data(&targets, move |_, selection, _| {
			selection.set(&selection.target(), 8i32, string.as_bytes());
		});
	}

	pub(crate) fn read_text(&self) -> Option<String> {
		let display = gdk::Display::default().unwrap();
		let clipboard = gtk::Clipboard::default(&display).unwrap();

		for target in &CLIPBOARD_TARGETS {
			let atom = Atom::intern(target);
			if let Some(selection) = clipboard.wait_for_contents(&atom) {
				return String::from_utf8(selection.data()).ok();
			}
		}

		None
	}
}
