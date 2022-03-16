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

#![allow(clippy::deprecated_semver)]

// The use of NSVisualEffectView comes from https://github.com/joboet/winit/tree/macos_blurred_background
// with a bit of rewrite by @youngsing to make it more like cocoa::appkit style.

/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/material>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NSVisualEffectMaterial {
	#[deprecated(
		since = "macOS 10.14",
		note = "A default material for the view's effectiveAppearance. You should instead choose an appropriate semantic material."
	)]
	AppearanceBased = 0,
	#[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
	Light = 1,
	#[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
	Dark = 2,
	#[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
	MediumLight = 8,
	#[deprecated(since = "macOS 10.14", note = "Use a semantic material instead.")]
	UltraDark = 9,

	// macOS 10.10+
	Titlebar = 3,
	Selection = 4,

	// macOS 10.11+
	Menu = 5,
	Popover = 6,
	Sidebar = 7,

	// macOS 10.14+
	HeaderView = 10,
	Sheet = 11,
	WindowBackground = 12,
	HudWindow = 13,
	FullScreenUI = 15,
	Tooltip = 17,
	ContentBackground = 18,
	UnderWindowBackground = 21,
	UnderPageBackground = 22
}

#[cfg(target_os = "macos")]
pub use internal::apply_vibrancy;

#[cfg(target_os = "macos")]
mod internal {
	use cocoa::{
		appkit::{
			NSAppKitVersionNumber, NSAppKitVersionNumber10_10, NSAppKitVersionNumber10_11, NSAutoresizingMaskOptions, NSView, NSViewHeightSizable,
			NSViewWidthSizable, NSWindow, NSWindowOrderingMode
		},
		base::{id, nil, BOOL},
		foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize}
	};
	use objc::{class, msg_send, sel, sel_impl};

	use super::NSVisualEffectMaterial;
	use crate::Error;

	#[allow(deprecated)]
	pub fn apply_vibrancy(window: id, appearance: NSVisualEffectMaterial) -> Result<(), Error> {
		unsafe {
			if NSAppKitVersionNumber < NSAppKitVersionNumber10_10 {
				eprintln!("\"NSVisualEffectView\" is only available on macOS 10.10 or newer.");
				return Err(Error::UnsupportedPlatformVersion("\"apply_vibrancy\" is only available on macOS 10.10 or newer."));
			}

			if !msg_send![class!(NSThread), isMainThread] {
				return Err(Error::NotMainThread("\"apply_vibrancy\" must be called on the main thread."));
			}

			let mut m = appearance;
			if appearance as u32 > 9 && NSAppKitVersionNumber < NSAppKitVersionNumber10_14 {
				m = NSVisualEffectMaterial::AppearanceBased;
			} else if appearance as u32 > 4 && NSAppKitVersionNumber < NSAppKitVersionNumber10_11 {
				m = NSVisualEffectMaterial::AppearanceBased;
			}

			let ns_view: id = window.contentView();
			let bounds = NSView::bounds(ns_view);

			let blurred_view = NSVisualEffectView::initWithFrame_(NSVisualEffectView::alloc(nil), bounds);
			blurred_view.autorelease();

			blurred_view.setMaterial_(m);
			blurred_view.setBlendingMode_(NSVisualEffectBlendingMode::BehindWindow);
			blurred_view.setState_(NSVisualEffectState::FollowsWindowActiveState);
			NSVisualEffectView::setAutoresizingMask_(blurred_view, NSViewWidthSizable | NSViewHeightSizable);

			let _: () = msg_send![ns_view, addSubview: blurred_view positioned: NSWindowOrderingMode::NSWindowBelow relativeTo: 0];
		}
		Ok(())
	}

	#[allow(non_upper_case_globals)]
	const NSAppKitVersionNumber10_14: f64 = 1671.0;

	// https://developer.apple.com/documentation/appkit/nsvisualeffectview/blendingmode
	#[allow(dead_code)]
	#[repr(u64)]
	#[derive(Clone, Copy, Debug, PartialEq)]
	enum NSVisualEffectBlendingMode {
		BehindWindow = 0,
		WithinWindow = 1
	}

	// https://developer.apple.com/documentation/appkit/nsvisualeffectview/state
	#[allow(dead_code)]
	#[repr(u64)]
	#[derive(Clone, Copy, Debug, PartialEq)]
	enum NSVisualEffectState {
		FollowsWindowActiveState = 0,
		Active = 1,
		Inactive = 2
	}

	// macOS 10.10+ - https://developer.apple.com/documentation/appkit/nsvisualeffectview
	#[allow(non_snake_case)]
	trait NSVisualEffectView: Sized {
		unsafe fn alloc(_: Self) -> id {
			msg_send![class!(NSVisualEffectView), alloc]
		}

		unsafe fn init(self) -> id;
		unsafe fn initWithFrame_(self, frameRect: NSRect) -> id;
		unsafe fn bounds(self) -> NSRect;
		unsafe fn frame(self) -> NSRect;
		unsafe fn setFrameSize(self, frameSize: NSSize);
		unsafe fn setFrameOrigin(self, frameOrigin: NSPoint);

		unsafe fn superview(self) -> id;
		unsafe fn removeFromSuperview(self);
		unsafe fn setAutoresizingMask_(self, autoresizingMask: NSAutoresizingMaskOptions);

		// API_AVAILABLE(macos(10.12));
		unsafe fn isEmphasized(self) -> BOOL;
		// API_AVAILABLE(macos(10.12));
		unsafe fn setEmphasized_(self, emphasized: BOOL);

		unsafe fn setMaterial_(self, material: NSVisualEffectMaterial);
		unsafe fn setState_(self, state: NSVisualEffectState);
		unsafe fn setBlendingMode_(self, mode: NSVisualEffectBlendingMode);
	}

	#[allow(non_snake_case)]
	impl NSVisualEffectView for id {
		unsafe fn init(self) -> id {
			msg_send![self, init]
		}

		unsafe fn initWithFrame_(self, frameRect: NSRect) -> id {
			msg_send![self, initWithFrame: frameRect]
		}

		unsafe fn bounds(self) -> NSRect {
			msg_send![self, bounds]
		}

		unsafe fn frame(self) -> NSRect {
			msg_send![self, frame]
		}

		unsafe fn setFrameSize(self, frameSize: NSSize) {
			msg_send![self, setFrameSize: frameSize]
		}

		unsafe fn setFrameOrigin(self, frameOrigin: NSPoint) {
			msg_send![self, setFrameOrigin: frameOrigin]
		}

		unsafe fn superview(self) -> id {
			msg_send![self, superview]
		}

		unsafe fn removeFromSuperview(self) {
			msg_send![self, removeFromSuperview]
		}

		unsafe fn setAutoresizingMask_(self, autoresizingMask: NSAutoresizingMaskOptions) {
			msg_send![self, setAutoresizingMask: autoresizingMask]
		}

		// API_AVAILABLE(macos(10.12));
		unsafe fn isEmphasized(self) -> BOOL {
			msg_send![self, isEmphasized]
		}

		// API_AVAILABLE(macos(10.12));
		unsafe fn setEmphasized_(self, emphasized: BOOL) {
			msg_send![self, setEmphasized: emphasized]
		}

		unsafe fn setMaterial_(self, material: NSVisualEffectMaterial) {
			msg_send![self, setMaterial: material]
		}

		unsafe fn setState_(self, state: NSVisualEffectState) {
			msg_send![self, setState: state]
		}

		unsafe fn setBlendingMode_(self, mode: NSVisualEffectBlendingMode) {
			msg_send![self, setBlendingMode: mode]
		}
	}
}
