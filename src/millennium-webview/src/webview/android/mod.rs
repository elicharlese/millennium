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

use crossbeam_channel::*;
use html5ever::{interface::QualName, namespace_url, ns, tendril::TendrilSink, LocalName};
use http::{
	header::{HeaderValue, CONTENT_SECURITY_POLICY, CONTENT_TYPE},
	Request, Response
};
use kuchiki::NodeRef;
use millennium_core::platform::android::ndk_glue::{
	jni::{
		errors::Error as JniError,
		objects::{GlobalRef, JClass, JObject},
		JNIEnv
	},
	ndk::looper::{FdEvent, ForeignLooper},
	PACKAGE
};
use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};

use super::{Rgba, WebContext, WebViewAttributes};
use crate::{application::window::Window, Result};

pub(crate) mod binding;
mod main_pipe;
use self::main_pipe::{CreateWebViewAttributes, MainPipe, WebViewMessage, MAIN_PIPE};

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

pub struct UnsafeRequestHandler(Box<dyn Fn(Request<Vec<u8>>) -> Option<Response<Vec<u8>>>>);
impl UnsafeRequestHandler {
	pub fn new(f: Box<dyn Fn(Request<Vec<u8>>) -> Option<Response<Vec<u8>>>>) -> Self {
		Self(f)
	}
}
unsafe impl Send for UnsafeRequestHandler {}
unsafe impl Sync for UnsafeRequestHandler {}

pub unsafe fn setup(env: JNIEnv, looper: &ForeignLooper, activity: GlobalRef) {
	// we must create the `WebChromeClient` here because it calls `registerForActivityResult`,
	// which gives a `LifecycleOwners must call register before they are STARTED.` error when
	// called outside the onCreate hook
	let rust_webchrome_client_class = find_my_class(env, activity.as_obj(), format!("{}/RustWebChromeClient", PACKAGE.get().unwrap())).unwrap();
	let webchrome_client = env
		.new_object(rust_webchrome_client_class, "(Landroidx/appcompat/app/AppCompatActivity;)V", &[activity.as_obj().into()])
		.unwrap();

	let mut main_pipe = MainPipe {
		env,
		activity,
		webview: None,
		webchrome_client: env.new_global_ref(webchrome_client).unwrap()
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

pub(crate) struct InnerWebView {
	#[allow(unused)]
	pub window: Rc<Window>
}

impl InnerWebView {
	pub fn new(
		window: Rc<Window>,
		attributes: WebViewAttributes,
		_pl_attrs: super::PlatformSpecificWebViewAttributes,
		_web_context: Option<&mut WebContext>
	) -> Result<Self> {
		let WebViewAttributes {
			url,
			initialization_scripts,
			ipc_handler,
			devtools,
			custom_protocols,
			background_color,
			transparent,
			..
		} = attributes;

		if let Some(u) = url {
			let mut url_string = String::from(u.as_str());
			let name = u.scheme();
			let schemes = custom_protocols.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>();
			if schemes.contains(&name) {
				url_string = u.as_str().replace(&format!("{}://", name), &format!("https://{}.", name))
			}

			MainPipe::send(WebViewMessage::CreateWebView(CreateWebViewAttributes {
				url: url_string,
				devtools,
				background_color,
				transparent
			}));
		}

		REQUEST_HANDLER.get_or_init(move || {
			UnsafeRequestHandler::new(Box::new(move |mut request| {
				if let Some(custom_protocol) = custom_protocols
					.iter()
					.find(|(name, _)| request.uri().to_string().starts_with(&format!("https://{}.", name)))
				{
					*request.uri_mut() = request
						.uri()
						.to_string()
						.replace(&format!("https://{}.", custom_protocol.0), &format!("{}://", custom_protocol.0))
						.parse()
						.unwrap();

					if let Ok(mut response) = (custom_protocol.1)(&request) {
						if response.headers().get(CONTENT_TYPE) == Some(&HeaderValue::from_static("text/html")) {
							if !initialization_scripts.is_empty() {
								let mut document = kuchiki::parse_html().one(String::from_utf8_lossy(&response.body()).into_owned());
								let csp = response.headers_mut().get_mut(CONTENT_SECURITY_POLICY);
								let mut hashes = Vec::new();
								with_html_head(&mut document, |head| {
									for script in &initialization_scripts {
										let script_el = NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
										script_el.append(NodeRef::new_text(script));

										head.prepend(script_el);
										if csp.is_some() {
											hashes.push(hash_script(script));
										}
									}
								});

								if let Some(csp) = csp {
									let csp_string = csp.to_str().unwrap().to_string();
									let csp_string = if csp_string.contains("script-src") {
										csp_string.replace("script-src", &format!("script-src {}", hashes.join(" ")))
									} else {
										format!("{} script-src {}", csp_string, hashes.join(" "))
									};
									*csp = HeaderValue::from_str(&csp_string).unwrap();
								}

								*response.body_mut() = document.to_string().as_bytes().to_vec();
							}
						}
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

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn open_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn close_devtools(&self) {}

	#[cfg(any(debug_assertions, feature = "devtools"))]
	pub fn is_devtools_open(&self) -> bool {
		false
	}

	pub fn zoom(&self, _scale_factor: f64) {}

	pub fn set_background_color(&self, background_color: Rgba) -> Result<()> {
		MainPipe::send(WebViewMessage::SetBackgroundColor(background_color));
		Ok(())
	}
}

#[derive(Clone, Copy)]
pub struct JniHandle;

impl JniHandle {
	/// Execute JNI code on the webview thread.
	///
	/// The passed function will be provided with the JNI evironment, Android activity, and WebView instance.
	pub fn exec<F>(&self, func: F)
	where
		F: FnOnce(JNIEnv, JObject, JObject) + Send + 'static
	{
		MainPipe::send(WebViewMessage::Jni(Box::new(func)));
	}
}

pub fn platform_webview_version() -> Result<String> {
	let (tx, rx) = bounded(1);
	MainPipe::send(WebViewMessage::GetWebViewVersion(tx));
	rx.recv().unwrap()
}

fn with_html_head<F: FnOnce(&NodeRef)>(document: &mut NodeRef, f: F) {
	if let Ok(ref node) = document.select_first("head") {
		f(node.as_node())
	} else {
		let node = NodeRef::new_element(QualName::new(None, ns!(html), LocalName::from("head")), None);
		f(&node);
		document.prepend(node)
	}
}

fn hash_script(script: &str) -> String {
	let mut hasher = Sha256::new();
	hasher.update(script);
	let hash = hasher.finalize();
	format!("'sha256-{}'", base64::encode(hash))
}

fn find_my_class<'a>(env: JNIEnv<'a>, activity: JObject<'a>, name: String) -> Result<JClass<'a>, JniError> {
	let class_name = env.new_string(name.replace('/', "."))?;
	let my_class = env
		.call_method(activity, "getAppClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[class_name.into()])?
		.l()?;
	Ok(my_class.into())
}
