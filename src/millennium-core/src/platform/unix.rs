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
	/// Whether to enable or disable the internal draw for transparent window.
	///
	/// When the tranparent attribute is enabled, `millennium-core` calls `connect_draw` and draws a transparent
	/// background. If you'd like to draw the background manually, set this to `false`. Default is `true`.
	fn with_transparent_draw(self, draw: bool) -> WindowBuilder;
	/// Whether to enable or disable the double buffered rendering of the window.
	///
	/// Default is `true`.
	fn with_double_buffered(self, double_buffered: bool) -> WindowBuilder;
	/// Whether to enable the RGBA visual for the window.
	///
	/// Default is `false`, but is always `true` if
	/// [`WindowAttributes::transparent`](crate::window::WindowAttributes::transparent) is `true`.
	fn with_rgba_visual(self, rgba_visual: bool) -> WindowBuilder;
	/// Wether to set this window as [app paintable](https://docs.gtk.org/gtk3/method.Widget.set_app_paintable.html).
	///
	/// Default is `false`, but is always `true` if
	/// [`WindowAttributes::transparent`](crate::window::WindowAttributes::transparent) is `true`.
	fn with_app_paintable(self, app_paintable: bool) -> WindowBuilder;
	/// Whether to enable processing the cursor moved event. The cursor move event is suited for native GUI frameworks
	/// and games, but it can occasionally block GTK's own pipeline. Turning this off can help GTK look smoother.
	///
	/// Default is `true`.
	fn with_cursor_moved_event(self, cursor_moved: bool) -> WindowBuilder;
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

	fn with_transparent_draw(mut self, draw: bool) -> WindowBuilder {
		self.platform_specific.auto_transparent = draw;
		self
	}

	fn with_double_buffered(mut self, double_buffered: bool) -> WindowBuilder {
		self.platform_specific.double_buffered = double_buffered;
		self
	}

	fn with_rgba_visual(mut self, rgba_visual: bool) -> WindowBuilder {
		self.platform_specific.rgba_visual = rgba_visual;
		self
	}

	fn with_app_paintable(mut self, app_paintable: bool) -> WindowBuilder {
		self.platform_specific.app_paintable = app_paintable;
		self
	}

	fn with_cursor_moved_event(mut self, cursor_moved: bool) -> WindowBuilder {
		self.platform_specific.cursor_moved = cursor_moved;
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

/// Additional methods on `EventLoopWindowTarget` that are specific to Unix.
pub trait EventLoopWindowTargetExtUnix {
	/// True if the `EventLoopWindowTarget` uses Wayland.
	fn is_wayland(&self) -> bool;

	/// True if the `EventLoopWindowTarget` uses X11.
	fn is_x11(&self) -> bool;

	fn xlib_xconnection(&self) -> Option<Arc<XConnection>>;

	// /// Returns a pointer to the `wl_display` object of wayland that is used by this
	// /// `EventLoopWindowTarget`.
	// ///
	// /// Returns `None` if the `EventLoop` doesn't use wayland (if it uses xlib for example).
	// ///
	// /// The pointer will become invalid when the winit `EventLoop` is destroyed.
	// fn wayland_display(&self) -> Option<*mut raw::c_void>;
}

impl<T> EventLoopWindowTargetExtUnix for EventLoopWindowTarget<T> {
	#[inline]
	fn is_wayland(&self) -> bool {
		self.p.is_wayland()
	}

	#[inline]
	fn is_x11(&self) -> bool {
		!self.p.is_wayland()
	}

	#[inline]
	fn xlib_xconnection(&self) -> Option<Arc<XConnection>> {
		if self.is_x11() {
			if let Ok(xconn) = XConnection::new(Some(x_error_callback)) {
				Some(Arc::new(xconn))
			} else {
				None
			}
		} else {
			None
		}
	}
}

unsafe extern "C" fn x_error_callback(_display: *mut x11::ffi::Display, event: *mut x11::ffi::XErrorEvent) -> c_int {
	let error = XError {
		// TODO: get the error text as description
		description: String::new(),
		error_code: (*event).error_code,
		request_code: (*event).request_code,
		minor_code: (*event).minor_code
	};

	error!("X11 error: {:#?}", error);

	0
}
