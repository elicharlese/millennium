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

use std::{fmt, io, iter::once, mem, os::windows::ffi::OsStrExt, path::Path, sync::Arc};

use windows::{
	core::PCWSTR,
	Win32::{
		Foundation::{HINSTANCE, HWND, LPARAM, WPARAM},
		System::LibraryLoader::*,
		UI::WindowsAndMessaging::*
	}
};

use crate::{dpi::PhysicalSize, icon::*};

impl Pixel {
	fn to_bgra(&mut self) {
		mem::swap(&mut self.r, &mut self.b);
	}
}

impl RgbaIcon {
	fn into_windows_icon(self) -> Result<WinIcon, BadIcon> {
		let mut rgba = self.rgba;
		let pixel_count = rgba.len() / PIXEL_SIZE;
		let mut and_mask = Vec::with_capacity(pixel_count);
		let pixels = unsafe { std::slice::from_raw_parts_mut(rgba.as_mut_ptr() as *mut Pixel, pixel_count) };
		for pixel in pixels {
			and_mask.push(pixel.a.wrapping_sub(std::u8::MAX)); // invert alpha channel
			pixel.to_bgra();
		}
		assert_eq!(and_mask.len(), pixel_count);
		let handle = unsafe {
			CreateIcon(
				HINSTANCE::default(),
				self.width as i32,
				self.height as i32,
				1,
				(PIXEL_SIZE * 8) as u8,
				and_mask.as_ptr() as *const u8,
				rgba.as_ptr() as *const u8
			)
		};
		Ok(WinIcon::from_handle(handle.map_err(|_| BadIcon::OsError(io::Error::last_os_error()))?))
	}
}

#[non_exhaustive]
#[derive(Debug)]
pub enum IconType {
	Small = ICON_SMALL as isize,
	Big = ICON_BIG as isize
}

#[derive(Debug)]
struct RaiiIcon {
	handle: HICON
}

#[derive(Clone)]
pub struct WinIcon {
	inner: Arc<RaiiIcon>
}

unsafe impl Send for WinIcon {}

impl WinIcon {
	pub fn as_raw_handle(&self) -> HICON {
		self.inner.handle
	}

	pub fn from_path<P: AsRef<Path>>(path: P, size: Option<PhysicalSize<u32>>) -> Result<Self, BadIcon> {
		let wide_path: Vec<u16> = path.as_ref().as_os_str().encode_wide().chain(once(0)).collect();

		// width / height of 0 along with LR_DEFAULTSIZE tells windows to load the
		// default icon size
		let (width, height) = size.map(Into::into).unwrap_or((0, 0));

		let handle =
			unsafe { LoadImageW(HINSTANCE::default(), PCWSTR(wide_path.as_ptr()), IMAGE_ICON, width as i32, height as i32, LR_DEFAULTSIZE | LR_LOADFROMFILE) }
				.map(|handle| HICON(handle.0));
		Ok(WinIcon::from_handle(handle.map_err(|_| BadIcon::OsError(io::Error::last_os_error()))?))
	}

	pub fn from_resource(resource_id: u16, size: Option<PhysicalSize<u32>>) -> Result<Self, BadIcon> {
		// width / height of 0 along with LR_DEFAULTSIZE tells windows to load the
		// default icon size
		let (width, height) = size.map(Into::into).unwrap_or((0, 0));
		let handle = unsafe {
			LoadImageW(
				GetModuleHandleW(PCWSTR::default()).unwrap_or_default(),
				PCWSTR(resource_id as usize as *const u16),
				IMAGE_ICON,
				width as i32,
				height as i32,
				LR_DEFAULTSIZE
			)
		}
		.map(|handle| HICON(handle.0));
		Ok(WinIcon::from_handle(handle.map_err(|_| BadIcon::OsError(io::Error::last_os_error()))?))
	}

	pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
		let rgba_icon = RgbaIcon::from_rgba(rgba, width, height)?;
		rgba_icon.into_windows_icon()
	}

	pub fn set_for_window(&self, hwnd: HWND, icon_type: IconType) {
		unsafe {
			SendMessageW(hwnd, WM_SETICON, WPARAM(icon_type as _), LPARAM(self.as_raw_handle().0));
		}
	}

	fn from_handle(handle: HICON) -> Self {
		Self { inner: Arc::new(RaiiIcon { handle }) }
	}
}

impl Drop for RaiiIcon {
	fn drop(&mut self) {
		unsafe { DestroyIcon(self.handle) };
	}
}

impl fmt::Debug for WinIcon {
	fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		(*self.inner).fmt(formatter)
	}
}

pub fn unset_for_window(hwnd: HWND, icon_type: IconType) {
	unsafe {
		SendMessageW(hwnd, WM_SETICON, WPARAM(icon_type as _), LPARAM(0));
	}
}
