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

use gdk::Display;

use crate::{
	dpi::{LogicalPosition, PhysicalPosition},
	error::ExternalError
};

#[inline]
pub fn cursor_position(is_wayland: bool) -> Result<PhysicalPosition<f64>, ExternalError> {
	if is_wayland {
		Ok((0, 0).into())
	} else {
		Display::default()
			.map(|d| (d.default_seat().and_then(|s| s.pointer()), d.default_group()))
			.map(|(p, g)| {
				p.map(|p| {
					let (_, x, y) = p.position_double();
					LogicalPosition::new(x, y).to_physical(g.scale_factor() as _)
				})
			})
			.map(|p| p.ok_or(ExternalError::Os(os_error!(super::OsError))))
			.ok_or(ExternalError::Os(os_error!(super::OsError)))?
	}
}
