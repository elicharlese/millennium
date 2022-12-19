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

#![allow(clippy::tabs_in_doc_comments)]

//! Make your Millennium windows vibrant.
//!
//! # Platform support
//!
//! - **Windows**: Yes!
//! - **macOS**: Yes!
//! - **Linux**: No, blur effects are controlled by the compositor and they can enable it for your app if they want.

use std::sync::Mutex;

use once_cell::sync::Lazy;
use thiserror::Error;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub use macos::NSVisualEffectMaterial;
#[cfg(target_os = "windows")]
pub use windows::{
	is_dwmsbt_supported as is_fluent_acrylic_supported, is_mica_attr_supported as is_mica_supported, is_swca_supported as is_acrylic_supported,
	is_win7 as is_blur_supported
};

/// a tuple of RGBA colors. Each value has a range of 0 to 255.
pub type VibrancyTint = (u8, u8, u8, u8);

/// <https://developer.apple.com/documentation/appkit/nsvisualeffectview/material>
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NSVisualEffectMaterial {
	/// A default material for the view's effectiveAppearance.
	#[deprecated = "Use a semantic material instead."]
	AppearanceBased = 0,
	#[deprecated = "Use a semantic material instead."]
	Light = 1,
	#[deprecated = "Use a semantic material instead."]
	Dark = 2,
	#[deprecated = "Use a semantic material instead."]
	MediumLight = 8,
	#[deprecated = "Use a semantic material instead."]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VibrancyEffect {
	/// Clear vibrancy effects.
	None,
	/// Windows 11 [Mica][mica] effect.
	///
	/// Supported in Windows 11 since build 22000 (21H2).
	///
	/// [mica]: https://learn.microsoft.com/en-us/windows/apps/design/style/mica
	Mica,
	/// Windows 11 [Fluent Acrylic][acrylic] effect.
	///
	/// Supported in Windows 11 since build 22523 (just before 22H2).
	///
	/// [acrylic]: https://learn.microsoft.com/en-us/windows/apps/design/style/acrylic
	FluentAcrylic,
	/// Windows 10 "acrylic" DWM blurbehind.
	///
	/// Supported in Windows 11 and 10 since build 17763 (October 2018 update or version 1809).
	/// Supports an optional 'tint' color to modify the color and transparency of the effect.
	///
	/// Performance may suffer when dragging the window on Windows 11 and potentially later versions of Windows 10.
	/// Please use [`VibrancyEffect::FluentAcrylic`] instead if the platform supports it.
	UnifiedAcrylic(Option<VibrancyTint>),
	/// Windows 7+ DWM blurbehind.
	///
	/// Supported since Windows 7.
	/// On Windows 10/11 builds later than 17763 (October 2018 update or version 1809), an optional 'tint' color is
	/// supported to modify the color and transparency of the effect. The tint option is not supported on Windows 7 and
	/// will do nothing.
	///
	/// Performance may suffer when dragging the window on Windows 11 and potentially later versions of Windows 10.
	/// Please use [`VibrancyEffect::FluentAcrylic`] instead if the platform supports it.
	Blurbehind(Option<VibrancyTint>),
	/// macOS visual effect materials.
	///
	/// Supported since macOS 10.10.
	///
	/// See Apple's [docs](https://developer.apple.com/documentation/appkit/nsvisualeffectview/material) and
	/// [`NSVisualEffectMaterial`] for more info.
	Vibrancy(NSVisualEffectMaterial)
}

#[derive(Debug, Error)]
pub enum VibrancyError {
	#[error("{0:?} is unsupported on this platform.")]
	EffectNotSupported(VibrancyEffect),
	#[error("\"apply_effect\" must be called on the main thread.")]
	NotMainThread(&'static str)
}

#[allow(unused)] // other platforms like Linux don't use this value
pub(crate) static LAST_EFFECT: Lazy<Mutex<VibrancyEffect>> = Lazy::new(|| Mutex::new(VibrancyEffect::None));

/// Sets the window's vibrancy effect.
///
/// Will return Err([`VibrancyError::EffectNotSupported`]) if the effect is not supported on this platform, except for
/// [`VibrancyEffect::None`], which is guaranteed to never error.
pub fn apply_effect(window: impl raw_window_handle::HasRawWindowHandle, effect: VibrancyEffect) -> Result<(), VibrancyError> {
	match window.raw_window_handle() {
		#[cfg(target_os = "windows")]
		raw_window_handle::RawWindowHandle::Win32(handle) => self::windows::apply_effect(handle.hwnd as _, effect),
		#[cfg(target_os = "macos")]
		raw_window_handle::RawWindowHandle::AppKit(handle) => self::macos::apply_effect(handle.ns_window as _, effect),
		_ if effect != VibrancyEffect::None => Err(VibrancyError::EffectNotSupported(effect)),
		_ => Ok(())
	}
}
