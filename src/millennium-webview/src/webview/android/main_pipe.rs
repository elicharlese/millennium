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

use std::os::unix::prelude::*;

use crossbeam_channel::*;
use millennium_core::platform::android::ndk_glue::{
	jni::{
		errors::Error as JniError,
		objects::{GlobalRef, JObject},
		JNIEnv
	},
	PACKAGE
};
use once_cell::sync::Lazy;

use super::find_my_class;
use crate::{webview::Rgba, Error};

static CHANNEL: Lazy<(Sender<WebViewMessage>, Receiver<WebViewMessage>)> = Lazy::new(|| bounded(8));
pub static MAIN_PIPE: Lazy<[RawFd; 2]> = Lazy::new(|| {
	let mut pipe: [RawFd; 2] = Default::default();
	unsafe { libc::pipe(pipe.as_mut_ptr()) };
	pipe
});

pub struct MainPipe<'a> {
	pub env: JNIEnv<'a>,
	pub activity: GlobalRef,
	pub webview: Option<GlobalRef>,
	pub webchrome_client: GlobalRef
}

impl MainPipe<'_> {
	pub fn send(message: WebViewMessage) {
		let size = std::mem::size_of::<bool>();
		if let Ok(()) = CHANNEL.0.send(message) {
			unsafe { libc::write(MAIN_PIPE[1], &true as *const _ as *const _, size) };
		}
	}

	pub fn recv(&mut self) -> Result<(), JniError> {
		let env = self.env;
		let activity = self.activity.as_obj();
		if let Ok(message) = CHANNEL.1.recv() {
			match message {
				WebViewMessage::CreateWebView(attrs) => {
					let CreateWebViewAttributes {
						url,
						devtools,
						transparent,
						background_color
					} = attrs;

					// Create webview
					let rust_webview_class = find_my_class(env, activity, format!("{}/RustWebView", PACKAGE.get().unwrap()))?;
					let webview = env.new_object(rust_webview_class, "(Landroid/content/Context;)V", &[activity.into()])?;

					// Enable Javascript
					let settings = env.call_method(webview, "getSettings", "()Landroid/webkit/WebSettings;", &[])?.l()?;
					env.call_method(settings, "setJavaScriptEnabled", "(Z)V", &[true.into()])?;

					// Load URL
					if let Ok(url) = env.new_string(url) {
						env.call_method(webview, "loadUrlMainThread", "(Ljava/lang/String;)V", &[url.into()])?;
					}

					// Enable devtools
					#[cfg(any(debug_assertions, feature = "devtools"))]
					env.call_static_method(rust_webview_class, "setWebContentsDebuggingEnabled", "(Z)V", &[devtools.into()])?;

					if transparent {
						set_background_color(env, webview, (0, 0, 0, 0))?;
					} else {
						if let Some(color) = background_color {
							set_background_color(env, webview, color)?;
						}
					}

					// Create and set webview client
					let rust_webview_client_class = find_my_class(env, activity, format!("{}/RustWebViewClient", PACKAGE.get().unwrap()))?;
					let webview_client = env.new_object(rust_webview_client_class, "()V", &[])?;
					env.call_method(webview, "setWebViewClient", "(Landroid/webkit/WebViewClient;)V", &[webview_client.into()])?;

					// set webchrome client
					env.call_method(webview, "setWebChromeClient", "(Landroid/webkit/WebChromeClient;)V", &[self.webchrome_client.as_obj().into()])?;

					// Add javascript interface (IPC)
					let ipc_class = find_my_class(env, activity, format!("{}/Ipc", PACKAGE.get().unwrap()))?;
					let ipc = env.new_object(ipc_class, "()V", &[])?;
					let ipc_str = env.new_string("ipc")?;
					env.call_method(webview, "addJavascriptInterface", "(Ljava/lang/Object;Ljava/lang/String;)V", &[ipc.into(), ipc_str.into()])?;

					// Set content view
					env.call_method(activity, "setContentView", "(Landroid/view/View;)V", &[webview.into()])?;
					let webview = env.new_global_ref(webview)?;
					self.webview = Some(webview);
				}
				WebViewMessage::Eval(script) => {
					if let Some(webview) = &self.webview {
						let s = env.new_string(script)?;
						env.call_method(
							webview.as_obj(),
							"evaluateJavascript",
							"(Ljava/lang/String;Landroid/webkit/ValueCallback;)V",
							&[s.into(), JObject::null().into()]
						)?;
					}
				}
				WebViewMessage::SetBackgroundColor(background_color) => {
					if let Some(webview) = &self.webview {
						set_background_color(env, webview.as_obj(), background_color)?;
					}
				}
				WebViewMessage::GetWebViewVersion(tx) => {
					match env
						.call_method(activity, "getVersion", "()Ljava/lang/String;", &[])
						.and_then(|v| v.l())
						.and_then(|s| env.get_string(s.into()))
					{
						Ok(version) => {
							tx.send(Ok(version.to_string_lossy().into())).unwrap();
						}
						Err(e) => tx.send(Err(e.into())).unwrap()
					}
				}
			}
		}
		Ok(())
	}
}

fn set_background_color<'a>(env: JNIEnv<'a>, webview: JObject<'a>, background_color: RGBA) -> Result<(), JniError> {
	let color_class = env.find_class("android/graphics/Color")?;
	let color = env.call_static_method(
		color_class,
		"argb",
		"(IIII)I",
		&[background_color.3.into(), background_color.0.into(), background_color.1.into(), background_color.2.into()]
	)?;
	env.call_method(webview, "setBackgroundColor", "(I)V", &[color])?;
	Ok(())
}

#[derive(Debug)]
pub enum WebViewMessage {
	CreateWebView(CreateWebViewAttributes),
	Eval(String),
	SetBackgroundColor(Rgba),
	GetWebViewVersion(Sender<Result<String, Error>>)
}

#[derive(Debug)]
pub struct CreateWebViewAttributes {
	pub url: String,
	pub devtools: bool,
	pub transparent: bool,
	pub background_color: Option<Rgba>
}
