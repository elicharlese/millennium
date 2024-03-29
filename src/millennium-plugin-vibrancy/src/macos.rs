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

use cocoa::{
	appkit::{
		NSAppKitVersionNumber, NSAppKitVersionNumber10_10, NSAppKitVersionNumber10_11, NSAutoresizingMaskOptions, NSView, NSViewHeightSizable,
		NSViewWidthSizable, NSWindow, NSWindowOrderingMode
	},
	base::{id, nil, BOOL},
	foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize}
};
use objc::{class, msg_send, sel, sel_impl};

use super::{NSVisualEffectMaterial, VibrancyError};

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

#[allow(deprecated)]
pub(crate) fn apply_effect(window: id, effect: VibrancyEffect) -> Result<(), VibrancyError> {
	let mut last_effect = LAST_EFFECT.lock().unwrap();

	match effect {
		VibrancyEffect::Vibrancy(appearance) => {
			unsafe {
				if NSAppKitVersionNumber < NSAppKitVersionNumber10_10 {
					eprintln!("\"NSVisualEffectView\" is only available on macOS 10.10 or newer.");
					return Err(VibrancyError::EffectNotSupported(effect));
				}

				if !msg_send![class!(NSThread), isMainThread] {
					return Err(VibrancyError::NotMainThread);
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
			};
		}
		VibrancyEffect::None => return Ok(()),
		_ => return Err(VibrancyError::EffectNotSupported(effect))
	}
	*last_effect = effect;

	Ok(())
}

fn remove_effect(_window: id, _effect: VibrancyEffect) {}
