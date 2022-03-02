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

//! iOS support
//!
//! # Building app
//! To build ios app you will need rustc built for this targets:
//!
//!  - armv7-apple-ios
//!  - armv7s-apple-ios
//!  - i386-apple-ios
//!  - aarch64-apple-ios
//!  - x86_64-apple-ios
//!
//! Then
//!
//! ```
//! cargo build --target=...
//! ```
//! The simplest way to integrate your app into xcode environment is to build it
//! as a static library. Wrap your main function and export it.
//!
//! ```rust, ignore
//! #[no_mangle]
//! pub extern fn start_app() {
//!     start_inner()
//! }
//!
//! fn start_inner() {
//!    ...
//! }
//! ```
//!
//! Compile project and then drag resulting .a into Xcode project. Add
//! millennium-core.h to xcode.
//!
//! ```ignore
//! void start_app();
//! ```
//!
//! Use start_app inside your xcode's main function.
//!
//!
//! # App lifecycle and events
//!
//! iOS environment is very different from other platforms and you must be very
//! careful with it's events. Familiarize yourself with
//! [app lifecycle](https://developer.apple.com/library/ios/documentation/UIKit/Reference/UIApplicationDelegate_Protocol/).
//!
//!
//! This is how those event are represented:
//!
//!  - applicationDidBecomeActive is Resumed
//!  - applicationWillResignActive is Suspended
//!  - applicationWillTerminate is LoopDestroyed
//!
//! Keep in mind that after LoopDestroyed event is received every attempt to
//! draw with opengl will result in segfault.
//!
//! Also note that app may not receive the LoopDestroyed event if suspended; it
//! might be SIGKILL'ed.

#![cfg(target_os = "ios")]

// TODO: (mtak-) UIKit requires main thread for virtually all function/method
// calls. This could be worked around in the future by using GCD (grand central
// dispatch) and/or caching of values like window size/position.
macro_rules! assert_main_thread {
    ($($t:tt)*) => {
        let is_main_thread: ::objc::runtime::BOOL = msg_send!(class!(NSThread), isMainThread);
        if is_main_thread == ::objc::runtime::NO {
            panic!($($t)*);
        }
    };
}

mod app_state;
mod clipboard;
mod event_loop;
mod ffi;
mod keycode;
mod monitor;
mod view;
mod window;

use std::fmt;

pub use self::{
	clipboard::Clipboard,
	event_loop::{EventLoop, EventLoopProxy, EventLoopWindowTarget},
	keycode::{keycode_from_scancode, keycode_to_scancode},
	monitor::{MonitorHandle, VideoMode},
	window::{PlatformSpecificWindowBuilderAttributes, Window, WindowId}
};
pub(crate) use crate::icon::NoIcon as PlatformIcon;
use crate::{
	accelerator::Accelerator,
	menu::{CustomMenuItem, MenuId, MenuItem, MenuType}
};

// todo: implement iOS keyboard event
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KeyEventExtra {}

// todo: implement iOS menubar
#[derive(Debug, Clone)]
pub struct MenuItemAttributes;
#[derive(Debug, Clone)]
pub struct Menu;

impl Default for Menu {
	fn default() -> Self {
		return Menu::new();
	}
}

impl Menu {
	pub fn new() -> Self {
		return Menu {};
	}
	pub fn new_popup_menu() -> Self {
		return Self::new();
	}
	pub fn add_item(
		&mut self,
		_menu_id: MenuId,
		_title: &str,
		_accelerator: Option<Accelerator>,
		_enabled: bool,
		_selected: bool,
		_menu_type: MenuType
	) -> CustomMenuItem {
		return CustomMenuItem(MenuItemAttributes {});
	}
	pub fn add_submenu(&mut self, _title: &str, _enabled: bool, _submenu: Menu) {}
	pub fn add_native_item(&mut self, _item: MenuItem, _menu_type: MenuType) -> Option<CustomMenuItem> {
		return None;
	}
}

impl MenuItemAttributes {
	pub fn id(self) -> MenuId {
		return MenuId::EMPTY;
	}
	pub fn set_enabled(&mut self, _is_enabled: bool) {}
	pub fn set_title(&mut self, _title: &str) {}
	pub fn set_selected(&mut self, _is_selected: bool) {}
	pub fn set_icon(&mut self, _icon: Vec<u8>) {}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId {
	uiscreen: ffi::id
}

impl DeviceId {
	pub unsafe fn dummy() -> Self {
		return DeviceId {
			uiscreen: std::ptr::null_mut()
		};
	}
}

unsafe impl Send for DeviceId {}
unsafe impl Sync for DeviceId {}

#[non_exhaustive]
#[derive(Debug)]
pub enum OsError {}

impl fmt::Display for OsError {
	fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
		return match self {
			_ => unreachable!()
		};
	}
}
