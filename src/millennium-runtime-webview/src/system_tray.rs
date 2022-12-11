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
	fmt,
	sync::{Arc, Mutex}
};

pub use millennium_runtime::{
	menu::{Menu, MenuEntry, MenuHash, MenuItem, MenuUpdate, Submenu, SystemTrayMenu, SystemTrayMenuEntry, SystemTrayMenuItem, TrayHandle},
	Icon, SystemTray, SystemTrayEvent, UserEvent
};
#[cfg(target_os = "macos")]
pub use millennium_webview::application::platform::macos::{CustomMenuItemExtMacOS, SystemTrayBuilderExtMacOS, SystemTrayExtMacOS};
pub use millennium_webview::application::{
	event::TrayEvent,
	event_loop::{EventLoopProxy, EventLoopWindowTarget},
	menu::{ContextMenu as MillenniumContextMenu, CustomMenuItem as MillenniumCustomMenuItem, MenuItem as MillenniumMenuItem},
	system_tray::{Icon as MillenniumTrayIcon, SystemTray as MillenniumSystemTray, SystemTrayBuilder},
	TrayId as MillenniumTrayId
};

use crate::{Error, Message, Result, TrayId, TrayMessage};

pub type GlobalSystemTrayEventHandler = Box<dyn Fn(TrayId, &SystemTrayEvent) + Send>;
pub type GlobalSystemTrayEventListeners = Arc<Mutex<Vec<Arc<GlobalSystemTrayEventHandler>>>>;
pub type SystemTrayEventHandler = Box<dyn Fn(&SystemTrayEvent) + Send>;
pub type SystemTrayEventListeners = Arc<Mutex<Vec<Arc<SystemTrayEventHandler>>>>;
pub type SystemTrayItems = Arc<Mutex<HashMap<u16, MillenniumCustomMenuItem>>>;

#[derive(Clone, Default)]
pub struct TrayContext {
	pub tray: Arc<Mutex<Option<MillenniumSystemTray>>>,
	pub listeners: SystemTrayEventListeners,
	pub items: SystemTrayItems
}

impl fmt::Debug for TrayContext {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("TrayContext").field("items", &self.items).finish()
	}
}

#[derive(Clone, Default)]
pub struct SystemTrayManager {
	pub trays: Arc<Mutex<HashMap<TrayId, TrayContext>>>,
	pub global_listeners: GlobalSystemTrayEventListeners
}

impl fmt::Debug for SystemTrayManager {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("SystemTrayManager").field("trays", &self.trays).finish()
	}
}

/// Wrapper around a [`millennium_webview::application::system_tray::Icon`] that can be created from a [`WindowIcon`].
pub struct TrayIcon(pub(crate) MillenniumTrayIcon);

impl TryFrom<Icon> for TrayIcon {
	type Error = Error;

	fn try_from(icon: Icon) -> std::result::Result<Self, Self::Error> {
		MillenniumTrayIcon::from_rgba(icon.rgba, icon.width, icon.height)
			.map(Self)
			.map_err(crate::icon_err)
	}
}

pub fn create_tray<T>(
	id: MillenniumTrayId,
	system_tray: SystemTray,
	event_loop: &EventLoopWindowTarget<T>
) -> crate::Result<(MillenniumSystemTray, HashMap<u16, MillenniumCustomMenuItem>)> {
	let icon = TrayIcon::try_from(system_tray.icon.expect("tray icon not set"))?;

	let mut items = HashMap::new();

	#[allow(unused_mut)]
	let mut builder = SystemTrayBuilder::new(icon.0, system_tray.menu.map(|menu| to_millennium_context_menu(&mut items, menu))).with_id(id);

	#[cfg(target_os = "macos")]
	{
		builder = builder
			.with_icon_as_template(system_tray.icon_as_template)
			.with_menu_on_left_click(system_tray.menu_on_left_click);

		if let Some(title) = system_tray.title {
			builder = builder.with_title(&title);
		}
	}

	let tray = builder.build(event_loop).map_err(|e| Error::SystemTray(Box::new(e)))?;

	Ok((tray, items))
}

#[derive(Debug, Clone)]
pub struct SystemTrayHandle<T: UserEvent> {
	pub(crate) id: TrayId,
	pub(crate) proxy: EventLoopProxy<super::Message<T>>
}

impl<T: UserEvent> TrayHandle for SystemTrayHandle<T> {
	fn set_icon(&self, icon: Icon) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::UpdateIcon(icon)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	fn set_menu(&self, menu: SystemTrayMenu) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::UpdateMenu(menu)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	fn update_item(&self, id: u16, update: MenuUpdate) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::UpdateItem(id, update)))
			.map_err(|_| Error::FailedToSendMessage)
	}
	#[cfg(target_os = "macos")]
	fn set_icon_as_template(&self, is_template: bool) -> millennium_runtime::Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::UpdateIconAsTemplate(is_template)))
			.map_err(|_| Error::FailedToSendMessage)
	}

	#[cfg(target_os = "macos")]
	fn set_title(&self, title: &str) -> millennium_runtime::Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::UpdateTitle(title.to_owned())))
			.map_err(|_| Error::FailedToSendMessage)
	}

	fn destroy(&self) -> Result<()> {
		self.proxy
			.send_event(Message::Tray(self.id, TrayMessage::Destroy))
			.map_err(|_| Error::FailedToSendMessage)
	}
}

impl From<SystemTrayMenuItem> for crate::MenuItemWrapper {
	fn from(item: SystemTrayMenuItem) -> Self {
		match item {
			SystemTrayMenuItem::Separator => Self(MillenniumMenuItem::Separator),
			_ => unimplemented!()
		}
	}
}

pub fn to_millennium_context_menu(custom_menu_items: &mut HashMap<MenuHash, MillenniumCustomMenuItem>, menu: SystemTrayMenu) -> MillenniumContextMenu {
	let mut tray_menu = MillenniumContextMenu::new();
	for item in menu.items {
		match item {
			SystemTrayMenuEntry::CustomItem(c) => {
				#[allow(unused_mut)]
				let mut item = tray_menu.add_item(crate::MenuItemAttributesWrapper::from(&c).0);
				#[cfg(target_os = "macos")]
				if let Some(native_image) = c.native_image {
					item.set_native_image(crate::NativeImageWrapper::from(native_image).0);
				}
				custom_menu_items.insert(c.id, item);
			}
			SystemTrayMenuEntry::NativeItem(i) => {
				tray_menu.add_native_item(crate::MenuItemWrapper::from(i).0);
			}
			SystemTrayMenuEntry::Submenu(submenu) => {
				tray_menu.add_submenu(&submenu.title, submenu.enabled, to_millennium_context_menu(custom_menu_items, submenu.inner));
			}
		}
	}
	tray_menu
}
