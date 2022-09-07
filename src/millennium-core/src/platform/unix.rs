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

#![cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]

use std::{os::raw::c_int, sync::Arc};

use self::x11::xdisplay::XConnection;
#[doc(hidden)]
pub use crate::platform_impl::x11;
pub use crate::platform_impl::{hit_test, EventLoop as UnixEventLoop};
use crate::{
	event_loop::{EventLoop, EventLoopWindowTarget},
	platform_impl::{x11::xdisplay::XError, Parent},
	window::{Window, WindowBuilder}
};

/// Additional methods on `Window` that are specific to Unix.
pub trait WindowExtUnix {
	/// Returns the `ApplicatonWindow` from gtk crate that is used by this
	/// window.
	fn gtk_window(&self) -> &gtk::ApplicationWindow;

	/// Whether to show the window icon in the taskbar or not.
	fn set_skip_taskbar(&self, skip: bool);
}

impl WindowExtUnix for Window {
	fn gtk_window(&self) -> &gtk::ApplicationWindow {
		&self.window.window
	}

	fn set_skip_taskbar(&self, skip: bool) {
		self.window.set_skip_taskbar(skip);
	}
}

pub trait WindowBuilderExtUnix {
	/// Whether to create the window icon with the taskbar icon or not.
	fn with_skip_taskbar(self, skip: bool) -> WindowBuilder;
	/// Set this window as a transient dialog for `parent`
	/// <https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/struct.Window.html#method.set_transient_for>
	fn with_transient_for(self, parent: gtk::ApplicationWindow) -> WindowBuilder;
}

impl WindowBuilderExtUnix for WindowBuilder {
	fn with_skip_taskbar(mut self, skip: bool) -> WindowBuilder {
		self.platform_specific.skip_taskbar = skip;
		self
	}

	fn with_transient_for(mut self, parent: gtk::ApplicationWindow) -> WindowBuilder {
		self.platform_specific.parent = Parent::ChildOf(parent);
		self
	}
}

/// Additional methods on `EventLoop` that are specific to Unix.
pub trait EventLoopExtUnix {
	/// Builds a new `EventLoop` on any thread.
	///
	/// This method bypasses the cross-platform compatibility requirement
	/// that `EventLoop` be created on the main thread.
	fn new_any_thread() -> Self
	where
		Self: Sized;
}

fn wrap_ev<T>(event_loop: UnixEventLoop<T>) -> EventLoop<T> {
	EventLoop {
		event_loop,
		_marker: std::marker::PhantomData
	}
}

impl<T> EventLoopExtUnix for EventLoop<T> {
	#[inline]
	fn new_any_thread() -> Self {
		wrap_ev(UnixEventLoop::new_any_thread())
	}
}
