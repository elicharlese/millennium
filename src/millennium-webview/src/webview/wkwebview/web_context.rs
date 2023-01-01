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

use std::borrow::Cow;

use http::{Request, Response};

use crate::{webview::web_context::WebContextData, Result};

#[derive(Debug)]
pub struct WebContextImpl {
	protocol_ptrs: Vec<*mut Box<dyn Fn(&Request<Vec<u8>>) -> Result<Response<Cow<'static, [u8]>>>>>
}

impl WebContextImpl {
	pub fn new(_data: &WebContextData) -> Self {
		Self { protocols: Vec::new() }
	}

	pub fn set_allows_automation(&mut self, _flag: bool) {}

	pub fn registered_protocols(&mut self, handler: *mut Box<dyn Fn(&Request<Vec<u8>>) -> Result<Response<Cow<'static, [u8]>>>>) {
		self.protocols.push(handler);
	}
}

impl Drop for WebContextImpl {
	fn drop(&mut self) {
		// We need to drop handler closures here
		unsafe {
			for ptr in self.protocols.iter() {
				if !ptr.is_null() {
					let _ = Box::from_raw(*ptr);
				}
			}
		}
	}
}
