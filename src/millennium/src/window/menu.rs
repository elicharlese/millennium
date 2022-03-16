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
	collections::HashMap,
	sync::{Arc, Mutex}
};

use millennium_macros::default_runtime;

use crate::{
	runtime::{
		menu::{MenuHash, MenuId, MenuIdRef, MenuUpdate},
		Dispatch
	},
	Runtime
};

/// The window menu event.
#[derive(Debug, Clone)]
pub struct MenuEvent {
	pub(crate) menu_item_id: MenuId
}

impl MenuEvent {
	/// The menu item id.
	pub fn menu_item_id(&self) -> MenuIdRef<'_> {
		&self.menu_item_id
	}
}

/// A handle to a system tray. Allows updating the context menu items.
#[default_runtime(crate::MillenniumWebview, millennium_webview)]
#[derive(Debug)]
pub struct MenuHandle<R: Runtime> {
	pub(crate) ids: Arc<Mutex<HashMap<MenuHash, MenuId>>>,
	pub(crate) dispatcher: R::Dispatcher
}

impl<R: Runtime> Clone for MenuHandle<R> {
	fn clone(&self) -> Self {
		Self {
			ids: self.ids.clone(),
			dispatcher: self.dispatcher.clone()
		}
	}
}

/// A handle to a system tray menu item.
#[default_runtime(crate::MillenniumWebview, millennium_webview)]
#[derive(Debug)]
pub struct MenuItemHandle<R: Runtime> {
	id: u16,
	dispatcher: R::Dispatcher
}

impl<R: Runtime> Clone for MenuItemHandle<R> {
	fn clone(&self) -> Self {
		Self {
			id: self.id,
			dispatcher: self.dispatcher.clone()
		}
	}
}

impl<R: Runtime> MenuHandle<R> {
	/// Gets a handle to the menu item that has the specified `id`.
	pub fn get_item(&self, id: MenuIdRef<'_>) -> MenuItemHandle<R> {
		for (raw, item_id) in self.ids.lock().unwrap().iter() {
			if item_id == id {
				return MenuItemHandle {
					id: *raw,
					dispatcher: self.dispatcher.clone()
				};
			}
		}
		panic!("item id not found")
	}

	/// Shows the menu.
	pub fn show(&self) -> crate::Result<()> {
		self.dispatcher.show_menu().map_err(Into::into)
	}

	/// Hides the menu.
	pub fn hide(&self) -> crate::Result<()> {
		self.dispatcher.hide_menu().map_err(Into::into)
	}

	/// Whether the menu is visible or not.
	pub fn is_visible(&self) -> crate::Result<bool> {
		self.dispatcher.is_menu_visible().map_err(Into::into)
	}

	/// Toggles the menu visibility.
	pub fn toggle(&self) -> crate::Result<()> {
		if self.is_visible()? { self.hide() } else { self.show() }
	}
}

impl<R: Runtime> MenuItemHandle<R> {
	/// Modifies the enabled state of the menu item.
	pub fn set_enabled(&self, enabled: bool) -> crate::Result<()> {
		self.dispatcher
			.update_menu_item(self.id, MenuUpdate::SetEnabled(enabled))
			.map_err(Into::into)
	}

	/// Modifies the title (label) of the menu item.
	pub fn set_title<S: Into<String>>(&self, title: S) -> crate::Result<()> {
		self.dispatcher
			.update_menu_item(self.id, MenuUpdate::SetTitle(title.into()))
			.map_err(Into::into)
	}

	/// Modifies the selected state of the menu item.
	pub fn set_selected(&self, selected: bool) -> crate::Result<()> {
		self.dispatcher
			.update_menu_item(self.id, MenuUpdate::SetSelected(selected))
			.map_err(Into::into)
	}

	#[cfg(target_os = "macos")]
	#[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
	pub fn set_native_image(&self, image: crate::NativeImage) -> crate::Result<()> {
		self.dispatcher
			.update_menu_item(self.id, MenuUpdate::SetNativeImage(image))
			.map_err(Into::into)
	}
}
