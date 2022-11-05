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

use std::{fs::File, io::BufWriter, path::Path};

use gdk_pixbuf::{Colorspace, Pixbuf};

use crate::window::BadIcon;

/// An icon used for the window titlebar, taskbar, etc.
#[derive(Debug, Clone)]
pub struct PlatformIcon {
	raw: Vec<u8>,
	width: i32,
	height: i32,
	row_stride: i32
}

impl From<PlatformIcon> for Pixbuf {
	fn from(icon: PlatformIcon) -> Self {
		Pixbuf::from_mut_slice(icon.raw, gdk_pixbuf::Colorspace::Rgb, true, 8, icon.width, icon.height, icon.row_stride)
	}
}

impl PlatformIcon {
	/// Creates an `Icon` from 32bpp RGBA data.
	///
	/// The length of `rgba` must be divisible by 4, and `width * height` must equal
	/// `rgba.len() / 4`. Otherwise, this will return a `BadIcon` error.
	pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Result<Self, BadIcon> {
		let row_stride = Pixbuf::calculate_rowstride(Colorspace::Rgb, true, 8, width as i32, height as i32);
		Ok(Self {
			raw: rgba,
			width: width as i32,
			height: height as i32,
			row_stride
		})
	}

	pub fn write_to_png(&self, path: impl AsRef<Path>) {
		let png = File::create(path).unwrap();
		let w = &mut BufWriter::new(png);

		let mut encoder = png::Encoder::new(w, self.width as _, self.height as _);
		encoder.set_color(png::ColorType::Rgba);
		encoder.set_depth(png::BitDepth::Eight);

		let mut writer = encoder.write_header().unwrap();
		writer.write_image_data(&self.raw).unwrap();
	}
}
