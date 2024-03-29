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
	dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
	monitor::{MonitorHandle as RootMonitorHandle, VideoMode as RootVideoMode}
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonitorHandle {
	pub(crate) monitor: gdk::Monitor
}

impl MonitorHandle {
	pub fn new(display: &gdk::Display, number: i32) -> Self {
		let monitor = display.monitor(number).unwrap();
		Self { monitor }
	}

	#[inline]
	pub fn name(&self) -> Option<String> {
		self.monitor.model().map(|s| s.as_str().to_string())
	}

	#[inline]
	pub fn size(&self) -> PhysicalSize<u32> {
		let rect = self.monitor.geometry();
		LogicalSize {
			width: rect.width() as u32,
			height: rect.height() as u32
		}
		.to_physical(self.scale_factor())
	}

	#[inline]
	pub fn position(&self) -> PhysicalPosition<i32> {
		let rect = self.monitor.geometry();
		LogicalPosition { x: rect.x(), y: rect.y() }.to_physical(self.scale_factor())
	}

	#[inline]
	pub fn scale_factor(&self) -> f64 {
		self.monitor.scale_factor() as f64
	}

	#[inline]
	pub fn video_modes(&self) -> Box<dyn Iterator<Item = RootVideoMode>> {
		Box::new(Vec::new().into_iter())
	}
}

unsafe impl Send for MonitorHandle {}
unsafe impl Sync for MonitorHandle {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode;

impl VideoMode {
	#[inline]
	pub fn size(&self) -> PhysicalSize<u32> {
		panic!("VideoMode is unsupported on Linux.")
	}

	#[inline]
	pub fn bit_depth(&self) -> u16 {
		panic!("VideoMode is unsupported on Linux.")
	}

	#[inline]
	pub fn refresh_rate(&self) -> u16 {
		panic!("VideoMode is unsupported on Linux.")
	}

	#[inline]
	pub fn monitor(&self) -> RootMonitorHandle {
		panic!("VideoMode is unsupported on Linux.")
	}
}

pub fn from_point(display: &Display, x: f64, y: f64) -> Option<MonitorHandle> {
	if let Some(monitor) = display.monitor_at_point(x as i32, y as i32) {
		(0..display.n_monitors())
			.map(|i| (i, display.monitor(i).unwrap()))
			.find(|cur| cur.1.geometry() == monitor.geometry())
			.map(|x| MonitorHandle::new(display, x.0))
	} else {
		None
	}
}
