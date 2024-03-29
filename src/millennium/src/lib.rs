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

//! Millennium is a framework for building tiny, blazing fast binaries for all
//! major desktop platforms. Developers can integrate any front-end framework
//! that compiles to HTML, JS and CSS for building their user interface.
//! The backend of the application is a rust-sourced binary with an API that the
//! front-end can interact with.
//!
//! # Cargo features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section) that can be enabled or disabled:
//!
//! - **millennium_webview** *(enabled by default)*: Enables the Millennium Webview runtime. Only disable it if you want
//!   a custom runtime.
//! - **dox**: Internal feature to generate Rust documentation without linking on Linux.
//! - **objc-exception**: Wrap each `msg_send!` in a @try/@catch and panics if an exception is caught, preventing
//!   Objective-C from unwinding into Rust.
//! - **isolation**: Enables the isolation pattern. Enabled by default if the `millennium > pattern > use` config option
//!   is set to `isolation` in the Millennium config file.
//! - **custom-protocol**: Feature managed by the Millennium CLI. When enabled, Millennium assumes a production
//!   environment instead of a development one.
//! - **updater**: Enables the application auto updater. Enabled by default if the `updater` config is defined in the
//!   Millennium config file.
//! - **devtools**: Enables the developer tools (Web inspector) and [`Window::open_devtools`]. Enabled by default on
//!   debug builds. On macOS it uses private APIs, so you can't enable it if your app will be published to the App
//!   Store.
//! - **shell-open-api**: Enables the [`api::shell`] module.
//! - **http-api**: Enables the [`api::http`] module.
//! - **http-multipart**: Adds support for `multipart/form-data` HTTP requests.
//! - **reqwest-client**: Uses `reqwest` as HTTP client on the `http` APIs. Improves performance, but increases the
//!   bundle size.
//! - **native-tls-vendored**: Compile and statically link to a vendored copy of OpenSSL (applies to the default HTTP
//!   client).
//! - **reqwest-native-tls-vendored**: Compile and statically link to a vendored copy of OpenSSL (applies to the
//!   `reqwest` HTTP client).
//! - **process-command-api**: Enables the [`api::process::Command`] APIs.
//! - **global-shortcut**: Enables the global shortcut APIs.
//! - **clipboard**: Enables the clipboard APIs.
//! - **process-relaunch-dangerous-allow-symlink-macos**: Allows the [`api::process::current_binary`] function to allow
//!   symlinks on macOS. **This is dangerous**, see the Security section in the function's documentation.
//! - **dialog**: Enables the [`api::dialog`] module.
//! - **notification**: Enables the [`api::notification`] module.
//! - **fs-extract-api**: Enables the [`api::file::Extract`] API.
//! - **cli**: Enables usage of `clap` for CLI argument parsing. Enabled by default if the `cli` config is defined on
//!   the Millennium config file.
//! - **system-tray**: Enables application system tray API. Enabled by default if the `systemTray` config is defined on
//!   the Millennium config file.
//! - **macos-private-api**: Enables features only available in **macOS**'s private APIs, currently the `transparent`
//!   window functionality and the `fullScreenEnabled` preference setting to `true`. Enabled by default if the
//!   `millennium > macosPrivateApi` config flag is set to `true` on the Millennium config file.
//! - **windows7-compat**: Enables compatibility with Windows 7 for the notifications API.
//! - **window-data-url**: Enables usage of data URLs on the webview.
//! - **compression**: Enables asset compression. You should only disable this if you want faster compile times in
//!   release builds, though it produces larger binaries.
//! - **icon-ico**: Adds support to set `.ico` window icons. Enables [`Icon::File`] and [`Icon::Raw`] variants.
//! - **icon-png**: Adds support to set `.png` window icons. Enables [`Icon::File`] and [`Icon::Raw`] variants.
//!
//! ## Cargo allowlist features
//!
//! The following are a list of [Cargo features](https://doc.rust-lang.org/stable/cargo/reference/manifest.html#the-features-section)
//! that enables commands for Millennium's API package. These features are automatically enabled by the Millennium CLI
//! based on the `allowlist` configuration in the Millennium config file.
//!
//! - **api-all**: Enables all API endpoints.
//!
//! ### Clipboard allowlist
//!
//! - **clipboard-all**: Enables all [Clipboard APIs](https://tauri.studio/en/docs/api/js/modules/clipboard/).
//! - **clipboard-read-text**: Enables the [`readText` API](https://tauri.studio/en/docs/api/js/modules/clipboard/#readtext).
//! - **clipboard-write-text**: Enables the [`writeText` API](https://tauri.studio/en/docs/api/js/modules/clipboard/#writetext).
//!
//! ### Dialog allowlist
//!
//! - **dialog-all**: Enables all [Dialog APIs](https://tauri.studio/en/docs/api/js/modules/dialog).
//! - **dialog-ask**: Enables the [`ask` API](https://tauri.studio/en/docs/api/js/modules/dialog#ask).
//! - **dialog-confirm**: Enables the [`confirm` API](https://tauri.studio/en/docs/api/js/modules/dialog#confirm).
//! - **dialog-message**: Enables the [`message` API](https://tauri.studio/en/docs/api/js/modules/dialog#message).
//! - **dialog-open**: Enables the [`open` API](https://tauri.studio/en/docs/api/js/modules/dialog#open).
//! - **dialog-save**: Enables the [`save` API](https://tauri.studio/en/docs/api/js/modules/dialog#save).
//!
//! ### Filesystem allowlist
//!
//! - **fs-all**: Enables all [Filesystem APIs](https://tauri.studio/en/docs/api/js/modules/fs).
//! - **fs-copy-file**: Enables the [`copyFile` API](https://tauri.studio/en/docs/api/js/modules/fs#copyfile).
//! - **fs-create-dir**: Enables the [`createDir` API](https://tauri.studio/en/docs/api/js/modules/fs#createdir).
//! - **fs-read-dir**: Enables the [`readDir` API](https://tauri.studio/en/docs/api/js/modules/fs#readdir).
//! - **fs-read-file**: Enables the [`readTextFile` API](https://tauri.studio/en/docs/api/js/modules/fs#readtextfile) and
//!   the [`readBinaryFile` API](https://tauri.studio/en/docs/api/js/modules/fs#readbinaryfile).
//! - **fs-remove-dir**: Enables the [`removeDir` API](https://tauri.studio/en/docs/api/js/modules/fs#removedir).
//! - **fs-remove-file**: Enables the [`removeFile` API](https://tauri.studio/en/docs/api/js/modules/fs#removefile).
//! - **fs-rename-file**: Enables the [`renameFile` API](https://tauri.studio/en/docs/api/js/modules/fs#renamefile).
//! - **fs-write-file**: Enables the [`writeFile` API](https://tauri.studio/en/docs/api/js/modules/fs#writefile) and the
//!   [`writeBinaryFile` API](https://tauri.studio/en/docs/api/js/modules/fs#writebinaryfile).
//!
//! ### Global shortcut allowlist
//!
//! - **global-shortcut-all**: Enables all [GlobalShortcut APIs](https://tauri.studio/en/docs/api/js/modules/globalShortcut).
//!
//! ### HTTP allowlist
//!
//! - **http-all**: Enables all [HTTP APIs](https://tauri.studio/en/docs/api/js/modules/http).
//! - **http-request**: Enables the [`request` APIs](https://tauri.studio/en/docs/api/js/classes/http.client/).
//!
//! ### Notification allowlist
//!
//! - **notification-all**: Enables all [Notification APIs](https://tauri.studio/en/docs/api/js/modules/notification).
//!
//! ### OS allowlist
//!
//! - **os-all**: Enables all [OS APIs](https://tauri.studio/en/docs/api/js/modules/os).
//!
//! ### Path allowlist
//!
//! - **path-all**: Enables all [Path APIs](https://tauri.studio/en/docs/api/js/modules/path).
//!
//! ### Process allowlist
//!
//! - **process-all**: Enables all [Process APIs](https://tauri.studio/en/docs/api/js/modules/process).
//! - **process-exit**: Enables the [`exit` API](https://tauri.studio/en/docs/api/js/modules/process#exit).
//! - **process-relaunch**: Enables the [`relaunch` API](https://tauri.studio/en/docs/api/js/modules/process#relaunch).
//!
//! ### Protocol allowlist
//!
//! - **protocol-all**: Enables all Protocol APIs.
//! - **protocol-asset**: Enables the `asset` custom protocol.
//!
//! ### Shell allowlist
//!
//! - **shell-all**: Enables all [Clipboard APIs](https://tauri.studio/en/docs/api/js/modules/shell).
//! - **shell-execute**: Enables [executing arbitrary programs](https://tauri.studio/en/docs/api/js/classes/shell.Command#constructor).
//! - **shell-sidecar**: Enables [executing a `sidecar` program](https://tauri.studio/en/docs/api/js/classes/shell.Command#sidecar).
//! - **shell-open**: Enables the [`open` API](https://tauri.studio/en/docs/api/js/modules/shell#open).
//!
//! ### Window allowlist
//!
//! - **window-all**: Enables all [Window APIs](https://tauri.studio/en/docs/api/js/modules/window).
//! - **window-create**: Enables the API used to [create new windows](https://tauri.studio/en/docs/api/js/classes/window.webviewwindow/).
//! - **window-center**: Enables the [`center` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#center).
//! - **window-request-user-attention**: Enables the [`requestUserAttention` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#requestuserattention).
//! - **window-set-resizable**: Enables the [`setResizable` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setresizable).
//! - **window-set-title**: Enables the [`setTitle` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#settitle).
//! - **window-maximize**: Enables the [`maximize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#maximize).
//! - **window-unmaximize**: Enables the [`unmaximize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#unmaximize).
//! - **window-minimize**: Enables the [`minimize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#minimize).
//! - **window-unminimize**: Enables the [`unminimize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#unminimize).
//! - **window-show**: Enables the [`show` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#show).
//! - **window-hide**: Enables the [`hide` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#hide).
//! - **window-close**: Enables the [`close` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#close).
//! - **window-set-decorations**: Enables the [`setDecorations` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setdecorations).
//! - **window-set-always-on-top**: Enables the [`setAlwaysOnTop` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setalwaysontop).
//! - **window-set-size**: Enables the [`setSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setsize).
//! - **window-set-min-size**: Enables the [`setMinSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setminsize).
//! - **window-set-max-size**: Enables the [`setMaxSize` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setmaxsize).
//! - **window-set-position**: Enables the [`setPosition` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setposition).
//! - **window-set-fullscreen**: Enables the [`setFullscreen` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setfullscreen).
//! - **window-set-focus**: Enables the [`setFocus` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setfocus).
//! - **window-set-icon**: Enables the [`setIcon` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#seticon).
//! - **window-set-skip-taskbar**: Enables the [`setSkipTaskbar` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setskiptaskbar).
//! - **window-set-cursor-grab**: Enables the [`setCursorGrab` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setcursorgrab).
//! - **window-set-cursor-visible**: Enables the [`setCursorVisible` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setcursorvisible).
//! - **window-set-cursor-icon**: Enables the [`setCursorIcon` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setcursoricon).
//! - **window-set-cursor-position**: Enables the [`setCursorPosition` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#setcursorposition).
//! - **window-start-dragging**: Enables the [`startDragging` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#startdragging).
//! - **window-print**: Enables the [`print` API](https://tauri.studio/en/docs/api/js/classes/window.WebviewWindow#print).

#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(doc_cfg, feature(doc_cfg))]

#[cfg(target_os = "macos")]
#[doc(hidden)]
pub use embed_plist;
/// The Millennium error enum.
pub use error::Error;
pub use millennium_macros::{command, generate_handler};
#[cfg(shell_scope)]
#[doc(hidden)]
pub use regex;

pub mod api;
pub(crate) mod app;
pub mod async_runtime;
pub mod command;
/// The Millennium API endpoints.
mod endpoints;
mod error;
mod event;
mod hooks;
mod manager;
mod pattern;
pub mod plugin;
pub mod window;
use millennium_runtime as runtime;
/// The allowlist scopes.
pub mod scope;
mod state;
#[cfg(updater)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
pub mod updater;

/// A Millennium [`Runtime`] wrapper around Millennium Webview.
#[cfg(feature = "millennium_webview")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "millennium_webview")))]
pub type MillenniumWebview = millennium_runtime_webview::MillenniumWebview<EventLoopMessage>;
pub use millennium_utils as utils;

/// `Result<T, ::millennium::Error>`
pub type Result<T> = std::result::Result<T, Error>;

/// A task to run on the main thread.
pub type SyncTask = Box<dyn FnOnce() + Send>;

use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use interprocess::local_socket::*;
/// Reads the config file at compile time and generates a [`Context`] based on
/// its content.
///
/// The default config file path is a `Millennium.toml` file inside the Cargo
/// manifest directory of the crate being built.
///
/// # Custom Config Path
///
/// You may pass a string literal to this macro to specify a custom path for the
/// Millennium config file. If the path is relative, it will be search for
/// relative to the Cargo manifest of the compiling crate.
///
/// # Note
///
/// This macro should not be called if you are using [`millennium-build`] to
/// generate the context from inside your build script as it will just cause
/// excess computations that will be discarded. Use either the
/// [`millennium-build`] method or this macro - not both.
pub use millennium_macros::generate_context;
// Export types likely to be used by the application.
pub use runtime::http;
#[cfg(target_os = "macos")]
#[cfg_attr(doc_cfg, doc(cfg(target_os = "macos")))]
pub use runtime::{menu::NativeImage, ActivationPolicy};
use serde::Serialize;
#[cfg(all(desktop, feature = "system-tray"))]
#[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "system-tray"))))]
pub use {
	self::app::tray::{SystemTray, SystemTrayEvent, SystemTrayHandle},
	self::runtime::menu::{SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu}
};
pub use {
	self::app::WindowMenuEvent,
	self::event::{Event, EventHandler},
	self::runtime::menu::{AboutMetadata, CustomMenuItem, Menu, MenuEntry, MenuItem, Submenu},
	self::utils::TitleBarStyle,
	self::window::menu::MenuEvent
};
pub use {
	self::app::{App, AppHandle, AssetResolver, Builder, CloseRequestApi, GlobalWindowEvent, PathResolver, RunEvent, WindowEvent},
	self::hooks::{
		Invoke, InvokeError, InvokeHandler, InvokeMessage, InvokePayload, InvokeResolver, InvokeResponder, InvokeResponse, OnPageLoad, PageLoadPayload,
		SetupHook
	},
	self::manager::Asset,
	self::runtime::{
		webview::WebviewAttributes,
		window::{
			dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Pixel, Position, Size},
			CursorIcon, FileDropEvent
		},
		DeviceEventFilter, RunIteration, UserAttentionType
	},
	self::state::{State, StateManager},
	self::utils::{
		assets::Assets,
		config::{Config, WindowUrl},
		Env, PackageInfo, Theme
	},
	self::window::{Monitor, Window, WindowBuilder},
	scope::*
};

#[cfg(feature = "clipboard")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "clipboard")))]
pub use self::runtime::ClipboardManager;
#[cfg(all(desktop, feature = "global-shortcut"))]
#[cfg_attr(doc_cfg, doc(cfg(all(desktop, feature = "global-shortcut"))))]
pub use self::runtime::GlobalShortcutManager;

/// Updater events.
#[cfg(updater)]
#[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
#[derive(Debug, Clone)]
pub enum UpdaterEvent {
	/// An update is available.
	UpdateAvailable {
		/// The update body.
		body: String,
		/// The update release date.
		date: Option<time::OffsetDateTime>,
		/// The update version.
		version: String
	},
	/// The update is pending and about to be downloaded.
	Pending,
	/// The update download received a progress event.
	DownloadProgress {
		/// The length of the chunk that was downloaded on this iteration. Does not accumulate with previous chunks.
		chunk_length: usize,
		/// The total download size if it is known.
		content_length: Option<u64>
	},
	/// The update has been downloaded and is ready to be installed.
	Downloaded,
	/// The update has been applied and the app is now up to date.
	Updated,
	/// The app is already up to date.
	AlreadyUpToDate,
	/// An error occurred while updating.
	Error(String)
}

#[cfg(updater)]
impl UpdaterEvent {
	pub(crate) fn status_message(self) -> &'static str {
		match self {
			Self::Pending => updater::EVENT_STATUS_PENDING,
			Self::Downloaded => updater::EVENT_STATUS_DOWNLOADED,
			Self::Updated => updater::EVENT_STATUS_SUCCESS,
			Self::AlreadyUpToDate => updater::EVENT_STATUS_UPTODATE,
			Self::Error(_) => updater::EVENT_STATUS_ERROR,
			_ => unreachable!()
		}
	}
}

/// The user event type.
#[derive(Debug, Clone)]
pub enum EventLoopMessage {
	/// Updater event.
	#[cfg(updater)]
	#[cfg_attr(doc_cfg, doc(cfg(feature = "updater")))]
	Updater(UpdaterEvent)
}

/// The webview runtime interface. A wrapper around [`runtime::Runtime`] with
/// the proper user event type associated.
pub trait Runtime: runtime::Runtime<EventLoopMessage> {}

impl<W: runtime::Runtime<EventLoopMessage>> Runtime for W {}

/// Include a [`Context`] that was generated by [`millennium-build`] inside your
/// build script.
///
/// You should either use [`millennium-build`] and this macro to include the
/// compile time generated code, or [`generate_context!`]. Do not use both at
/// the same time, as they generate the same code and will cause excess
/// computations that will be discarded.
#[macro_export]
macro_rules! millennium_build_context {
	() => {
		include!(concat!(env!("OUT_DIR"), "/millennium-build-context.rs"))
	};
}

pub use pattern::Pattern;

/// An icon definition.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Icon {
	/// Icon from file path.
	#[cfg(any(feature = "icon-ico", feature = "icon-png"))]
	#[cfg_attr(doc_cfg, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
	File(std::path::PathBuf),
	/// Icon from raw RGBA bytes. Width and height is parsed at runtime.
	#[cfg(any(feature = "icon-ico", feature = "icon-png"))]
	#[cfg_attr(doc_cfg, doc(cfg(any(feature = "icon-ico", feature = "icon-png"))))]
	Raw(Vec<u8>),
	/// Icon from raw RGBA bytes.
	Rgba {
		/// RGBA bytes of the icon image.
		rgba: Vec<u8>,
		/// Icon width.
		width: u32,
		/// Icon height.
		height: u32
	}
}

impl TryFrom<Icon> for runtime::Icon {
	type Error = Error;

	fn try_from(icon: Icon) -> Result<Self> {
		#[allow(irrefutable_let_patterns)]
		if let Icon::Rgba { rgba, width, height } = icon {
			Ok(Self { rgba, width, height })
		} else {
			#[cfg(not(any(feature = "icon-ico", feature = "icon-png")))]
			panic!("unexpected Icon variant");
			#[cfg(any(feature = "icon-ico", feature = "icon-png"))]
			{
				let bytes = match icon {
					Icon::File(p) => std::fs::read(p)?,
					Icon::Raw(r) => r,
					Icon::Rgba { .. } => unreachable!()
				};
				let extension = infer::get(&bytes).expect("could not determine icon extension").extension();
				match extension {
					#[cfg(feature = "icon-ico")]
					"ico" => {
						let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes))?;
						let entry = &icon_dir.entries()[0];
						Ok(Self {
							rgba: entry.decode()?.rgba_data().to_vec(),
							width: entry.width(),
							height: entry.height()
						})
					}
					#[cfg(feature = "icon-png")]
					"png" => {
						let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
						let mut reader = decoder.read_info()?;
						let mut buffer = Vec::new();
						while let Ok(Some(row)) = reader.next_row() {
							buffer.extend(row.data());
						}
						Ok(Self {
							rgba: buffer,
							width: reader.info().width,
							height: reader.info().height
						})
					}
					_ => panic!(
						"image extension `{extension}` not supported; please file a feature request. PNG or ICO icons are supported via the `icon-png` and `icon-ico` feature flags respectively."
					)
				}
			}
		}
	}
}

/// User supplied data required inside of a Millennium application.
///
/// # Stability
/// This is the output of the [`generate_context`] macro, and is not considered
/// part of the stable API. Unless you know what you are doing and are prepared
/// for this type to have breaking changes, do not create it yourself.
pub struct Context<A: Assets> {
	pub(crate) config: Config,
	pub(crate) assets: Arc<A>,
	pub(crate) default_window_icon: Option<Icon>,
	pub(crate) app_icon: Option<Vec<u8>>,
	pub(crate) system_tray_icon: Option<Icon>,
	pub(crate) package_info: PackageInfo,
	pub(crate) _info_plist: (),
	pub(crate) pattern: Pattern,
	#[cfg(shell_scope)]
	pub(crate) shell_scope: scope::ShellScopeConfig
}

impl<A: Assets> fmt::Debug for Context<A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut d = f.debug_struct("Context");
		d.field("config", &self.config)
			.field("default_window_icon", &self.default_window_icon)
			.field("app_icon", &self.app_icon)
			.field("system_tray_icon", &self.system_tray_icon)
			.field("package_info", &self.package_info)
			.field("pattern", &self.pattern);
		#[cfg(shell_scope)]
		d.field("shell_scope", &self.shell_scope);
		d.finish()
	}
}

impl<A: Assets> Context<A> {
	/// The config the application was prepared with.
	#[inline(always)]
	pub fn config(&self) -> &Config {
		&self.config
	}

	/// A mutable reference to the config the application was prepared with.
	#[inline(always)]
	pub fn config_mut(&mut self) -> &mut Config {
		&mut self.config
	}

	/// The assets to be served directly by Millennium.
	#[inline(always)]
	pub fn assets(&self) -> Arc<A> {
		self.assets.clone()
	}

	/// A mutable reference to the assets to be served directly by Millennium.
	#[inline(always)]
	pub fn assets_mut(&mut self) -> &mut Arc<A> {
		&mut self.assets
	}

	/// The default window icon Millennium should use when creating windows.
	#[inline(always)]
	pub fn default_window_icon(&self) -> Option<&Icon> {
		self.default_window_icon.as_ref()
	}

	/// A mutable reference to the default window icon Millennium should use
	/// when creating windows.
	#[inline(always)]
	pub fn default_window_icon_mut(&mut self) -> &mut Option<Icon> {
		&mut self.default_window_icon
	}

	/// The icon to use on the system tray UI.
	#[inline(always)]
	pub fn system_tray_icon(&self) -> Option<&Icon> {
		self.system_tray_icon.as_ref()
	}

	/// A mutable reference to the icon to use on the system tray UI.
	#[inline(always)]
	pub fn system_tray_icon_mut(&mut self) -> &mut Option<Icon> {
		&mut self.system_tray_icon
	}

	/// Package information.
	#[inline(always)]
	pub fn package_info(&self) -> &PackageInfo {
		&self.package_info
	}

	/// A mutable reference to the package information.
	#[inline(always)]
	pub fn package_info_mut(&mut self) -> &mut PackageInfo {
		&mut self.package_info
	}

	/// The application pattern.
	#[inline(always)]
	pub fn pattern(&self) -> &Pattern {
		&self.pattern
	}

	/// The scoped shell commands, where the `HashMap` key is the name each
	/// configuration.
	#[cfg(shell_scope)]
	#[inline(always)]
	pub fn allowed_commands(&self) -> &scope::ShellScopeConfig {
		&self.shell_scope
	}

	/// Create a new [`Context`] from the minimal required items.
	#[inline(always)]
	#[allow(clippy::too_many_arguments)]
	pub fn new(
		config: Config,
		assets: Arc<A>,
		default_window_icon: Option<Icon>,
		app_icon: Option<Vec<u8>>,
		system_tray_icon: Option<Icon>,
		package_info: PackageInfo,
		info_plist: (),
		pattern: Pattern,
		#[cfg(shell_scope)] shell_scope: scope::ShellScopeConfig
	) -> Self {
		Self {
			config,
			assets,
			default_window_icon,
			app_icon,
			system_tray_icon,
			package_info,
			_info_plist: info_plist,
			pattern,
			#[cfg(shell_scope)]
			shell_scope
		}
	}
}

// TODO: expand these docs
/// Manages a running application.
pub trait Manager<R: Runtime>: sealed::ManagerBase<R> {
	/// The application handle associated with this manager.
	fn app_handle(&self) -> AppHandle<R> {
		self.managed_app_handle()
	}

	/// The [`Config`] the manager was created with.
	fn config(&self) -> Arc<Config> {
		self.manager().config()
	}

	/// Emits a event to all windows.
	fn emit_all<S: Serialize + Clone>(&self, event: &str, payload: S) -> Result<()> {
		self.manager().emit_filter(event, None, payload, |_| true)
	}

	/// Emits an event to a window with the specified label.
	fn emit_to<S: Serialize + Clone>(&self, label: &str, event: &str, payload: S) -> Result<()> {
		self.manager().emit_filter(event, None, payload, |w| label == w.label())
	}

	/// Listen to a global event.
	fn listen_global<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
	where
		F: Fn(Event) + Send + 'static
	{
		self.manager().listen(event.into(), None, handler)
	}

	/// Listen to a global event only once.
	fn once_global<F>(&self, event: impl Into<String>, handler: F) -> EventHandler
	where
		F: FnOnce(Event) + Send + 'static
	{
		self.manager().once(event.into(), None, handler)
	}

	/// Trigger a global event.
	fn trigger_global(&self, event: &str, data: Option<String>) {
		self.manager().trigger(event, None, data)
	}

	/// Remove an event listener.
	fn unlisten(&self, handler_id: EventHandler) {
		self.manager().unlisten(handler_id)
	}

	/// Fetch a single window from the manager.
	fn get_window(&self, label: &str) -> Option<Window<R>> {
		self.manager().get_window(label)
	}

	/// Fetch all managed windows.
	fn windows(&self) -> HashMap<String, Window<R>> {
		self.manager().windows()
	}

	/// Add `state` to the state managed by the application.
	///
	/// This method can be called any number of times as long as each call
	/// refers to a different `T`.
	///
	/// Managed state can be retrieved by any command handler via the
	/// [`State`](crate::State) guard. In particular, if a value of type
	/// `T` is managed by Millennium, adding `State<T>` to the list of arguments
	/// in a command handler instructs Millennium to retrieve the managed value.
	///
	/// # Panics
	///
	/// Panics if state of type `T` is already being managed.
	///
	/// # Mutability
	///
	/// Since the managed state is global and must be [`Send`] + [`Sync`],
	/// mutations can only happen through interior mutability:
	///
	/// ```rust,no_run
	/// use std::{collections::HashMap, sync::Mutex};
	///
	/// use millennium::State;
	/// // here we use Mutex to achieve interior mutability
	/// struct Storage {
	/// 	store: Mutex<HashMap<u64, String>>
	/// }
	/// struct Connection;
	/// struct DbConnection {
	/// 	db: Mutex<Option<Connection>>
	/// }
	///
	/// #[millennium::command]
	/// fn connect(connection: State<DbConnection>) {
	/// 	// initialize the connection, mutating the state with interior mutability
	/// 	*connection.db.lock().unwrap() = Some(Connection {});
	/// }
	///
	/// #[millennium::command]
	/// fn storage_insert(key: u64, value: String, storage: State<Storage>) {
	/// 	// mutate the storage behind the Mutex
	/// 	storage.store.lock().unwrap().insert(key, value);
	/// }
	///
	/// millennium::Builder::default()
	/// 	.manage(Storage { store: Default::default() })
	/// 	.manage(DbConnection { db: Default::default() })
	/// 	.invoke_handler(millennium::generate_handler![connect, storage_insert])
	/// 	// on an actual app, remove the string argument
	/// 	.run(millennium::generate_context!("test/fixture/Millennium.toml"))
	/// 	.expect("error while running millennium application");
	/// ```
	///
	/// # Examples
	///
	/// ```rust,no_run
	/// use millennium::State;
	///
	/// struct MyInt(isize);
	/// struct MyString(String);
	///
	/// #[millennium::command]
	/// fn int_command(state: State<MyInt>) -> String {
	/// 	format!("The stateful int is: {}", state.0)
	/// }
	///
	/// #[millennium::command]
	/// fn string_command<'r>(state: State<'r, MyString>) {
	/// 	println!("state: {}", state.inner().0);
	/// }
	///
	/// millennium::Builder::default()
	/// 	.manage(MyInt(10))
	/// 	.manage(MyString("Hello, managed state!".to_string()))
	/// 	.invoke_handler(millennium::generate_handler![int_command, string_command])
	/// 	// on an actual app, remove the string argument
	/// 	.run(millennium::generate_context!("test/fixture/Millennium.toml"))
	/// 	.expect("error while running Millennium application");
	/// ```
	fn manage<T>(&self, state: T) -> bool
	where
		T: Send + Sync + 'static
	{
		self.manager().state().set(state)
	}

	/// Retrieves the managed state for the type `T`.
	///
	/// # Panics
	///
	/// Panics if the state for the type `T` has not been previously
	/// [managed](Self::manage). Use [try_state](Self::try_state) for a
	/// non-panicking version.
	fn state<T>(&self) -> State<'_, T>
	where
		T: Send + Sync + 'static
	{
		self.manager()
			.inner
			.state
			.try_get()
			.expect("state() called before manage() for given type")
	}

	/// Attempts to retrieve the managed state for the type `T`.
	///
	/// Returns `Some` if the state has previously been [managed](Self::manage).
	/// Otherwise, returns `None`.
	fn try_state<T>(&self) -> Option<State<'_, T>>
	where
		T: Send + Sync + 'static
	{
		self.manager().inner.state.try_get()
	}

	/// Gets the managed [`Env`].
	fn env(&self) -> Env {
		self.state::<Env>().inner().clone()
	}

	/// Gets the scope for the filesystem APIs.
	fn fs_scope(&self) -> FsScope {
		self.state::<Scopes>().inner().fs.clone()
	}

	/// Gets the scope for the asset protocol.
	#[cfg(protocol_asset)]
	fn asset_protocol_scope(&self) -> FsScope {
		self.state::<Scopes>().inner().asset_protocol.clone()
	}

	/// Gets the scope for the shell execute APIs.
	#[cfg(shell_scope)]
	fn shell_scope(&self) -> ShellScope {
		self.state::<Scopes>().inner().shell.clone()
	}
}

/// Prevent implementation details from leaking out of the [`Manager`] trait.
pub(crate) mod sealed {
	use super::Runtime;
	use crate::{app::AppHandle, manager::WindowManager};

	/// A running [`Runtime`] or a dispatcher to it.
	pub enum RuntimeOrDispatch<'r, R: Runtime> {
		/// Reference to the running [`Runtime`].
		Runtime(&'r R),

		/// Handle to the running [`Runtime`].
		RuntimeHandle(R::Handle),

		/// A dispatcher to the running [`Runtime`].
		Dispatch(R::Dispatcher)
	}

	/// Managed handle to the application runtime.
	pub trait ManagerBase<R: Runtime> {
		/// The manager behind the [`Managed`] item.
		fn manager(&self) -> &WindowManager<R>;
		fn runtime(&self) -> RuntimeOrDispatch<'_, R>;
		fn managed_app_handle(&self) -> AppHandle<R>;
	}
}

/// Requests a single instance lock for the application. This can be used to ensure that only one instance of your app
/// is open at a time.
///
/// Returns `Ok(true)` if the lock was acquired (this is the first instance of the app), and `Ok(false)` if the lock was
/// not acquired (this is the second instance of the app).
///
/// `on_second_instance` will be called when a second instance of the app is launched with the instance's command line
/// arguments, current working directory, and process ID.
pub fn request_single_instance_lock<F>(id: &str, on_second_instance: F) -> Result<bool>
where
	F: Fn(Vec<&str>, PathBuf, u32) + Send + 'static
{
	use std::io::{prelude::*, BufReader};

	#[cfg(target_os = "windows")]
	let path: String = id.into();
	#[cfg(not(target_os = "windows"))]
	let path = format!("/tmp/{id}.lock");

	let _path = path.clone();
	if let Ok(listener) = LocalSocketListener::bind(_path) {
		std::thread::spawn(move || {
			for conn in listener.incoming().flatten() {
				let mut conn = BufReader::new(conn);
				let mut buffer = String::new();
				conn.read_line(&mut buffer).unwrap();

				let mut args = buffer.split('\r').collect::<Vec<_>>();
				let pid: u32 = args.pop().unwrap().trim_end().parse().unwrap();
				let cwd = args.pop().unwrap();
				on_second_instance(args, PathBuf::from(cwd), pid);
			}
		});
		Ok(true)
	} else {
		let mut conn = LocalSocketStream::connect(path)?;
		let args = std::env::args().collect::<Vec<_>>();
		let cwd = std::env::current_dir()?;
		let pid = std::process::id();
		conn.write_all(format!("{}\r{}\r{}\n", args.join("\r"), cwd.to_str().unwrap(), pid).as_bytes())?;
		Ok(false)
	}
}

/// Returns `true` if the application is running under ARM64 translation.
///
/// # Platform-specific
///
/// - **Linux**: Unsupported; returns false.
pub fn running_under_arm64_translation() -> bool {
	#[cfg(not(any(target_os = "macos", target_os = "windows")))]
	{
		false
	}
	#[cfg(target_os = "macos")]
	{
		use std::{
			ffi::{c_void, CString},
			os::raw::c_char
		};

		use libc::sysctlbyname;

		let name = CString::new("sysctl.proc_translated").unwrap();
		let mut proc_translated = 0i32;
		let mut size = std::mem::size_of_val(&proc_translated);
		if unsafe { sysctlbyname(name.as_ptr() as *const c_char, &mut proc_translated as *mut _ as *mut c_void, &mut size, std::ptr::null_mut(), 0) } == -1 {
			false
		} else {
			proc_translated == 1
		}
	}
	#[cfg(target_os = "windows")]
	{
		use windows::{
			core::PCSTR,
			Win32::{
				Foundation::{BOOL, HANDLE},
				System::{
					LibraryLoader::{GetModuleHandleA, GetProcAddress},
					SystemInformation::IMAGE_FILE_MACHINE_ARM64,
					Threading::GetCurrentProcess
				}
			}
		};

		type IsWow64Process2 = unsafe extern "system" fn(HANDLE, *mut u16, *mut u16) -> BOOL;

		if let Ok(handle) = unsafe { GetModuleHandleA(PCSTR::from_raw("kernel32.dll".as_ptr())) } {
			if let Some(is_wow64_process2) = unsafe { GetProcAddress(handle, PCSTR::from_raw("IsWow64Process2".as_ptr())) } {
				let is_wow64_process2: IsWow64Process2 = unsafe { std::mem::transmute(is_wow64_process2) };
				let mut process_arch: u16 = 0;
				let mut system_arch: u16 = 0;
				if unsafe { is_wow64_process2(GetCurrentProcess(), &mut process_arch, &mut system_arch) }.as_bool() {
					return system_arch == IMAGE_FILE_MACHINE_ARM64.0;
				}
			}
		}

		false
	}
}

/// Utilities for unit testing on Millennium applications.
#[cfg(test)]
pub mod test;

#[cfg(test)]
mod tests {
	use std::{env::var, path::PathBuf};

	use cargo_toml::Manifest;
	use once_cell::sync::OnceCell;

	static MANIFEST: OnceCell<Manifest> = OnceCell::new();
	const CHECKED_FEATURES: &str = include_str!(concat!(env!("OUT_DIR"), "/checked_features"));

	fn get_manifest() -> &'static Manifest {
		MANIFEST.get_or_init(|| {
			let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());

			Manifest::from_path(manifest_dir.join("Cargo.toml")).expect("failed to parse Cargo manifest")
		})
	}

	#[test]
	fn aliased_features_exist() {
		let checked_features = CHECKED_FEATURES.split(',');
		let manifest = get_manifest();
		for checked_feature in checked_features {
			if !manifest.features.iter().any(|(f, _)| f == checked_feature) {
				panic!("Feature {checked_feature} was checked in the alias build step but it does not exist in Cargo.toml");
			}
		}
	}

	#[test]
	fn all_allowlist_features_are_aliased() {
		let manifest = get_manifest();
		let all_modules = manifest
			.features
			.iter()
			.find(|(f, _)| f.as_str() == "api-all")
			.map(|(_, enabled)| enabled)
			.expect("api-all feature must exist");

		let checked_features = CHECKED_FEATURES.split(',').collect::<Vec<&str>>();
		assert!(checked_features.contains(&"api-all"), "`api-all` is not aliased");

		// features that look like an allowlist feature, but are not
		let allowed = [
			"fs-extract-api",
			"http-api",
			"http-multipart",
			"process-command-api",
			"process-relaunch-dangerous-allow-symlink-macos",
			"window-data-url"
		];

		for module_all_feature in all_modules {
			let module = module_all_feature.replace("-all", "");
			assert!(checked_features.contains(&module_all_feature.as_str()), "`{module}` is not aliased");

			let module_prefix = format!("{module}-");
			// we assume that module features are the ones that start with `<module>-`
			// though it's not 100% accurate, we have an allowed list to fix it
			let module_features = manifest.features.keys().filter(|f| f.starts_with(&module_prefix));
			for module_feature in module_features {
				assert!(allowed.contains(&module_feature.as_str()) || checked_features.contains(&module_feature.as_str()), "`{module_feature}` is not aliased");
			}
		}
	}
}

#[cfg(test)]
mod test_utils {
	use proptest::prelude::*;

	pub fn assert_send<T: Send>() {}
	pub fn assert_sync<T: Sync>() {}

	#[allow(dead_code)]
	pub fn assert_not_allowlist_error<T>(res: anyhow::Result<T>) {
		if let Err(e) = res {
			assert!(!e.to_string().contains("not on the allowlist"));
		}
	}

	proptest! {
		#![proptest_config(ProptestConfig::with_cases(10000))]
		#[test]
		// check to see if spawn executes a function.
		fn check_spawn_task(task in "[a-z]+") {
			// create dummy task function
			let dummy_task = async move {
				format!("{task}-run-dummy-task");
			};
			// call spawn
			crate::async_runtime::spawn(dummy_task);
		}
	}
}
