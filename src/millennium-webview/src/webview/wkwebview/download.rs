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

use std::{path::PathBuf, ptr::null_mut, rc::Rc};

use cocoa::base::id;
use libc::c_void;
use objc::{
	declare::ClassDecl,
	runtime::{Object, Sel}
};

use super::NSString;

pub(crate) unsafe fn set_download_delegate(webview: *mut Object, download_delegate: *mut Object) {
	(*webview).set_ivar("DownloadDelegate", download_delegate as *mut _ as *mut c_void);
}

unsafe fn get_download_delegate(this: &mut Object) -> *mut objc::runtime::Object {
	let delegate: *mut c_void = *this.get_ivar("DownloadDelegate");
	delegate as *mut Object
}

// Download action handler
extern "C" fn navigation_download_action(this: &mut Object, _: Sel, _: id, _: id, download: id) {
	unsafe {
		let delegate = get_download_delegate(this);
		let _: () = msg_send![download, setDelegate: delegate];
	}
}

// Download response handler
extern "C" fn navigation_download_response(this: &mut Object, _: Sel, _: id, _: id, download: id) {
	unsafe {
		let delegate = get_download_delegate(this);
		let _: () = msg_send![download, setDelegate: delegate];
	}
}

pub(crate) unsafe fn add_download_methods(decl: &mut ClassDecl) {
	decl.add_ivar::<*mut c_void>("DownloadDelegate");

	decl.add_method(sel!(webView:navigationAction:didBecomeDownload:), navigation_download_action as extern "C" fn(&mut Object, Sel, id, id, id));

	decl.add_method(sel!(webView:navigationResponse:didBecomeDownload:), navigation_download_response as extern "C" fn(&mut Object, Sel, id, id, id));
}

pub extern "C" fn download_policy(this: &Object, _: Sel, download: id, _: id, suggested_path: id, handler: id) {
	unsafe {
		let request: id = msg_send![download, originalRequest];
		let url: id = msg_send![request, URL];
		let url: id = msg_send![url, absoluteString];
		let url = NSString(url);
		let path = NSString(suggested_path);
		let mut path = PathBuf::from(path.to_str());
		let handler = handler as *mut block::Block<(id,), c_void>;

		let function = this.get_ivar::<*mut c_void>("started");
		if !function.is_null() {
			let function = &mut *(*function as *mut Box<dyn for<'s> FnMut(String, &mut PathBuf) -> bool>);
			match (function)(url.to_str().to_string(), &mut path) {
				true => {
					let nsurl: id = msg_send![class!(NSURL), fileURLWithPath: NSString::new(&path.display().to_string()) isDirectory: false];
					(*handler).call((nsurl,))
				}
				false => (*handler).call((null_mut(),))
			};
		} else {
			log::warn!("WebView instance is dropped! This navigation handler shouldn't be called.");
			(*handler).call((null_mut(),));
		}
	}
}

pub extern "C" fn download_did_finish(this: &Object, _: Sel, download: id) {
	unsafe {
		let function = this.get_ivar::<*mut c_void>("completed");
		let original_request: id = msg_send![download, originalRequest];
		let url: id = msg_send![original_request, URL];
		let url: id = msg_send![url, absoluteString];
		let url = NSString(url).to_str().to_string();
		if !function.is_null() {
			let function = &mut *(*function as *mut Rc<dyn for<'s> Fn(String, Option<PathBuf>, bool)>);
			function(url, None, true);
		}
	}
}

pub extern "C" fn download_did_fail(this: &Object, _: Sel, download: id, error: id, _: id) {
	unsafe {
		let description: id = msg_send![error, localizedDescription];
		let description = NSString(description).to_str().to_string();
		let original_request: id = msg_send![download, originalRequest];
		let url: id = msg_send![original_request, URL];
		let url: id = msg_send![url, absoluteString];
		let url = NSString(url).to_str().to_string();

		#[cfg(debug_assertions)]
		eprintln!("Download failed with error: {}", description);

		let function = this.get_ivar::<*mut c_void>("completed");
		if !function.is_null() {
			let function = &mut *(*function as *mut Rc<dyn for<'s> Fn(String, Option<PathBuf>, bool)>);
			function(url, None, false);
		}
	}
}
