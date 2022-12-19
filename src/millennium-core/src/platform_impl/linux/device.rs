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

use std::{
	os::raw::{c_int, c_uchar},
	ptr
};

use x11_dl::{xinput2, xlib};

use super::keycode_from_scancode;
use crate::{
	event::{DeviceEvent, ElementState, RawKeyEvent},
	event_loop::EventLoopWindowTarget
};

/// Spawn device event thread. Only works on X11 since Wayland doesn't have such global events.
pub fn spawn<T>(window_target: &EventLoopWindowTarget<T>, device_tx: glib::Sender<DeviceEvent>) {
	if !window_target.p.is_wayland() {
		std::thread::spawn(move || unsafe {
			let xlib = xlib::Xlib::open().unwrap();
			let xinput2 = xinput2::XInput2::open().unwrap();
			let display = (xlib.XOpenDisplay)(ptr::null());
			let root = (xlib.XDefaultRootWindow)(display);
			let mask = xinput2::XI_RawKeyPressMask | xinput2::XI_RawKeyReleaseMask;
			let mut event_mask = xinput2::XIEventMask {
				deviceid: xinput2::XIAllMasterDevices,
				mask: &mask as *const _ as *mut c_uchar,
				mask_len: std::mem::size_of_val(&mask) as c_int
			};
			(xinput2.XISelectEvents)(display, root, &mut event_mask as *mut _, 1);

			#[allow(clippy::uninit_assumed_init)]
			let mut event: xlib::XEvent = std::mem::MaybeUninit::uninit().assume_init();
			loop {
				(xlib.XNextEvent)(display, &mut event);

				// XFilterEvent tells us when an event has been discarded by the input method.
				// Specifically, this involves all of the KeyPress events in compose/pre-edit sequences,
				// along with an extra copy of the KeyRelease events. This also prevents backspace and
				// arrow keys from being detected twice.
				if xlib::True == {
					(xlib.XFilterEvent)(&mut event, {
						let xev: &xlib::XAnyEvent = event.as_ref();
						xev.window
					})
				} {
					continue;
				}

				let event_type = event.get_type();
				if event_type == xlib::GenericEvent {
					let mut xev = event.generic_event_cookie;
					if (xlib.XGetEventData)(display, &mut xev) == xlib::True {
						match xev.evtype {
							xinput2::XI_RawKeyPress | xinput2::XI_RawKeyRelease => {
								let xev: &xinput2::XIRawEvent = &*(xev.data as *const _);
								let physical_key = keycode_from_scancode(xev.detail as u32);
								let state = match xev.evtype {
									xinput2::XI_RawKeyPress => ElementState::Pressed,
									xinput2::XI_RawKeyRelease => ElementState::Released,
									_ => unreachable!()
								};

								let event = RawKeyEvent { physical_key, state };

								if let Err(e) = device_tx.send(DeviceEvent::Key(event)) {
									log::info!("Failed to send device event {} since receiver is closed, closing X11 thread along with it", e);
									break;
								}
							}
							_ => {}
						}
					}
				}
			}
		});
	}
}
