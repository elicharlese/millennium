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

use std::rc::Rc;

use millennium_core::platform::android::ndk_glue::{
	jni::{objects::GlobalRef, JNIEnv},
	ndk::looper::{FdEvent, ForeignLooper}
};
use once_cell::sync::OnceCell;

use crate::{
	application::window::Window,
	http::{Request as HttpRequest, Response as HttpResponse},
	Result
};

pub(crate) mod binding;
mod main_pipe;
use self::main_pipe::{MainPipe, WebViewMessage, MAIN_PIPE};

#[macro_export]
macro_rules! android_binding {
	($domain:ident, $package:ident, $main: ident) => {
		android_binding!($domain, $package, $main, ::millennium_webview)
	};
	($domain:ident, $package:ident, $main: ident, $millennium_webview: path) => {
		use $millennium_webview::{
			application::{android_binding as core_android_binding, android_fn, platform::android::ndk_glue::*},
			webview::prelude::*
		};
		core_android_binding!($domain, $package, setup, $main);
		android_fn!($domain, $package, RustWebChromeClient, runInitializationScripts);
		android_fn!($domain, $package, RustWebViewClient, handleRequest, JObject, jobject);
		android_fn!($domain, $package, Ipc, ipc, JString);
	};
}

pub static IPC: OnceCell<UnsafeIpc> = OnceCell::new();
pub static REQUEST_HANDLER: OnceCell<UnsafeRequestHandler> = OnceCell::new();

pub struct UnsafeIpc(Box<dyn Fn(&Window, String)>, Rc<Window>);
impl UnsafeIpc {
	pub fn new(f: Box<dyn Fn(&Window, String)>, w: Rc<Window>) -> Self {
		Self(f, w)
	}
}
unsafe impl Send for UnsafeIpc {}
unsafe impl Sync for UnsafeIpc {}

pub struct UnsafeRequestHandler(Box<dyn Fn(HttpRequest) -> Option<HttpResponse>>);
impl UnsafeRequestHandler {
	pub fn new(f: Box<dyn Fn(HttpRequest) -> Option<HttpResponse>>) -> Self {
		Self(f)
	}
}
unsafe impl Send for UnsafeRequestHandler {}
unsafe impl Sync for UnsafeRequestHandler {}

pub unsafe fn setup(env: JNIEnv, looper: &ForeignLooper, activity: GlobalRef) {
	let mut main_pipe = MainPipe {
		env,
		activity,
		initialization_scripts: vec![],
		webview: None
	};

	looper
		.add_fd_with_callback(MAIN_PIPE[0], FdEvent::INPUT, move |_| {
			let size = std::mem::size_of::<bool>();
			let mut wake = false;
			if libc::read(MAIN_PIPE[0], &mut wake as *mut _ as *mut _, size) == size as libc::ssize_t {
				match main_pipe.recv() {
					Ok(_) => true,
					Err(_) => false
				}
			} else {
				false
			}
		})
		.unwrap();
}

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
			UnsafeRequestHandler::new(Box::new(move |mut request| {
				if let Some(custom_protocol) = custom_protocols
					.iter()
					.find(|(name, _)| request.uri().starts_with(&format!("https://{}.", name)))
				{
					*request.uri_mut() = request
						.uri()
						.replace(&format!("https://{}.", custom_protocol.0), &format!("{}://", custom_protocol.0));

					if let Ok(response) = (custom_protocol.1)(&request) {
						return Some(response);
					}
				}

				None
			}))
		});

		let w = window.clone();
		if let Some(i) = ipc_handler {
			IPC.get_or_init(move || UnsafeIpc::new(Box::new(i), w));
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
