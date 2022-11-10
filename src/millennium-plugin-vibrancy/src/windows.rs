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

#![cfg(target_os = "windows")]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(unused_unsafe)]

use std::ffi::c_void;

use once_cell::sync::Lazy;
pub use windows_sys::Win32::{
	Foundation::{BOOL, FARPROC, HWND},
	Graphics::{
		Dwm::{
			DwmEnableBlurBehindWindow, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE, DWM_BB_ENABLE,
			DWM_BLURBEHIND
		},
		Gdi::HRGN
	},
	System::{
		LibraryLoader::{GetProcAddress, LoadLibraryA},
		SystemInformation::OSVERSIONINFOW
	},
	UI::Controls::MARGINS
};

use crate::{VibrancyEffect, VibrancyError, VibrancyTint, LAST_EFFECT};

type WINDOWCOMPOSITIONATTRIB = u32;

const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = 1029i32;
const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = 38i32;

#[repr(C)]
struct ACCENT_POLICY {
	AccentState: u32,
	AccentFlags: u32,
	GradientColor: u32,
	AnimationId: u32
}

#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
	Attrib: WINDOWCOMPOSITIONATTRIB,
	pvData: *mut c_void,
	cbData: usize
}

#[derive(PartialEq, Eq)]
#[repr(C)]
enum ACCENT_STATE {
	ACCENT_DISABLED = 0,
	ACCENT_ENABLE_BLURBEHIND = 3,
	ACCENT_ENABLE_ACRYLICBLURBEHIND = 4
}

#[allow(unused)]
#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
	DWMSBT_DISABLE = 1,
	DWMSBT_MAINWINDOW = 2,      // Mica
	DWMSBT_TRANSIENTWINDOW = 3, // Acrylic
	DWMSBT_TABBEDWINDOW = 4     // Tabbed Mica
}

fn get_function_impl(library: &str, function: &str) -> Option<FARPROC> {
	assert_eq!(library.chars().last(), Some('\0'));
	assert_eq!(function.chars().last(), Some('\0'));

	let module = unsafe { LoadLibraryA(library.as_ptr()) };
	if module == 0 {
		return None;
	}
	Some(unsafe { GetProcAddress(module, function.as_ptr()) })
}

macro_rules! get_function {
	($lib:expr, $func:ident, $type:ty) => {
		get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0')).map(|f| unsafe { std::mem::transmute::<FARPROC, $type>(f) })
	};
}

static WVER: Lazy<(u32, u32, u32)> = Lazy::new(|| {
	let RtlGetVersion = get_function!("ntdll.dll", RtlGetVersion, unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32).unwrap();
	let mut vi = OSVERSIONINFOW {
		dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32,
		dwMajorVersion: 0,
		dwMinorVersion: 0,
		dwBuildNumber: 0,
		dwPlatformId: 0,
		szCSDVersion: [0; 128]
	};
	unsafe { (RtlGetVersion)(&mut vi as _) };
	(vi.dwMajorVersion, vi.dwMinorVersion, vi.dwBuildNumber)
});

#[inline(always)]
pub fn is_win7() -> bool {
	WVER.0 > 6 || (WVER.0 == 6 && WVER.1 == 1)
}
#[inline(always)]
pub fn is_swca_supported() -> bool {
	WVER.2 >= 17763
}
#[inline(always)]
pub fn is_mica_attr_supported() -> bool {
	WVER.2 >= 22000
}
#[inline(always)]
pub fn is_dwmsbt_supported() -> bool {
	WVER.2 >= 22523
}

unsafe fn set_accent_policy(hwnd: HWND, accent_state: ACCENT_STATE, color: Option<VibrancyTint>) {
	if let Some(SetWindowCompositionAttribute) =
		get_function!("user32.dll", SetWindowCompositionAttribute, unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL)
	{
		let mut color = color.unwrap_or_default();

		let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
		if is_acrylic && color.3 == 0 {
			// acrylic doesn't like to have 0 alpha
			color.3 = 1;
		}

		let mut policy = ACCENT_POLICY {
			AccentState: accent_state as _,
			AccentFlags: if is_acrylic { 0 } else { 2 },
			GradientColor: (color.0 as u32) | (color.1 as u32) << 8 | (color.2 as u32) << 16 | (color.3 as u32) << 24,
			AnimationId: 0
		};
		let mut data = WINDOWCOMPOSITIONATTRIBDATA {
			Attrib: 0x13,
			pvData: &mut policy as *mut _ as _,
			cbData: std::mem::size_of_val(&policy)
		};
		SetWindowCompositionAttribute(hwnd, &mut data as *mut _ as _);
	}
}

pub(crate) fn apply_effect(hwnd: HWND, effect: VibrancyEffect) -> Result<(), VibrancyError> {
	let mut last_effect = LAST_EFFECT.lock().unwrap();
	if *last_effect != effect && *last_effect != VibrancyEffect::None {
		remove_effect(hwnd, &last_effect);
	}

	match effect {
		VibrancyEffect::None => remove_effect(hwnd, &last_effect),
		VibrancyEffect::Mica if is_dwmsbt_supported() || is_mica_attr_supported() => {
			unsafe {
				if is_dwmsbt_supported() {
					DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as *const _ as _, 4);
				} else if is_mica_attr_supported() {
					DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
				}
			};
		}
		VibrancyEffect::FluentAcrylic if is_dwmsbt_supported() => {
			unsafe { DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as *const _ as _, 4) };
		}
		VibrancyEffect::UnifiedAcrylic(tint) if is_swca_supported() => {
			unsafe { set_accent_policy(hwnd, ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND, tint) };
		}
		VibrancyEffect::Blurbehind(tint) if is_swca_supported() || is_win7() => {
			if is_swca_supported() {
				unsafe {
					set_accent_policy(hwnd, ACCENT_STATE::ACCENT_ENABLE_BLURBEHIND, tint);
				}
			} else if is_win7() {
				unsafe {
					let _ = DwmEnableBlurBehindWindow(
						hwnd,
						&DWM_BLURBEHIND {
							dwFlags: DWM_BB_ENABLE,
							fEnable: true.into(),
							hRgnBlur: HRGN::default(),
							fTransitionOnMaximized: 0
						}
					);
				}
			}
		}
		_ => return Err(VibrancyError::EffectNotSupported(effect))
	}
	*last_effect = effect;
	Ok(())
}

fn remove_effect(hwnd: HWND, effect: &VibrancyEffect) {
	match effect {
		VibrancyEffect::Mica => unsafe {
			if is_dwmsbt_supported() {
				DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4);
			} else if is_mica_attr_supported() {
				DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4);
			}
		},
		VibrancyEffect::FluentAcrylic if is_dwmsbt_supported() => {
			unsafe { DwmSetWindowAttribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _, 4) };
		}
		VibrancyEffect::UnifiedAcrylic(_) if is_swca_supported() => {
			unsafe { set_accent_policy(hwnd, ACCENT_STATE::ACCENT_DISABLED, None) };
		}
		VibrancyEffect::Blurbehind(_) => {
			if is_swca_supported() {
				unsafe {
					set_accent_policy(hwnd, ACCENT_STATE::ACCENT_DISABLED, None);
				}
			} else if is_win7() {
				unsafe {
					let _ = DwmEnableBlurBehindWindow(
						hwnd,
						&DWM_BLURBEHIND {
							dwFlags: DWM_BB_ENABLE,
							fEnable: false.into(),
							hRgnBlur: HRGN::default(),
							fTransitionOnMaximized: 0
						}
					);
				}
			}
		}
		_ => ()
	};
}
