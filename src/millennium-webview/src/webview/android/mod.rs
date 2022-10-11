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

use std::{rc::Rc, str::FromStr};

use millennium_core::android::ndk_glue::*;

use crate::{
	application::window::Window,
	http::{method::Method, Request as HttpRequest, RequestParts},
	Result
};

pub struct InnerWebView {
	pub window: Rc<Window>
}

impl InnerWebView {
	pub fn new(window: Rc<Window>, attributes: WebViewAttributes, _web_context: Option<&mut WebContext>) -> Result<Self> {
		let WebViewAttributes {
			url,
			initialization_scripts,
			ipc_handler,
			devtools,
			custom_protocols,
			..
		} = attributes;

		if let Some(u) = url {
			let mut url_string = String::from(u.as_str());
			let name = u.scheme();
			let schemes = custom_protocols.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>();
			if schemes.contains(&name) {
				url_string = u.as_str().replace(&format!("{}://", name), &format!("https://{}.", name))
			}

			MainPipe::send(WebViewMessage::CreateWebView(url_string, initialization_scripts, devtools));
		}

		REQUEST_HANDLER.get_or_init(move || {
			UnsafeRequestHandler::new(Box::new(move |request: WebResourceRequest| {
				if let Some(custom_protocol) = custom_protocols
					.iter()
					.find(|(name, _)| request.url.starts_with(&format!("https://{}.", name)))
				{
					let path = request
						.url
						.replace(&format!("https://{}.", custom_protocol.0), &format!("{}://", custom_protocol.0));

					let request = HttpRequest {
						head: RequestParts {
							method: Method::from_str(&request.method).unwrap_or(Method::GET),
							uri: path,
							headers: request.headers
						},
						body: Vec::new()
					};

					if let Ok(response) = (custom_protocol.1)(&request) {
						return Some(WebResourceResponse {
							status: response.head.status,
							headers: response.head.headers,
							mimetype: response.head.mimetype,
							body: response.body
						});
					}
				}

				None
			}))
		});

		let w = window.clone();
		if let Some(i) = ipc_handler {
			IPC.get_or_init(move || UnsafeIpc::new(Box::into_raw(Box::new(i)) as *mut _, w));
		}

		Ok(Self { window })
	}

	pub fn print(&self) {}

	pub fn eval(&self, js: &str) -> Result<()> {
		MainPipe::send(WebViewMessage::Eval(js.into()));
		Ok(())
	}

	pub fn focus(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn open_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn close_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn is_devtools_open(&self) -> bool {
		false
	}

	pub fn zoom(&self, _scale_factor: f64) {}
}

pub fn platform_webview_version() -> Result<String> {
	todo!()
}
