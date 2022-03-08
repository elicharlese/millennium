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

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use millennium_utils::assets::AssetKey;
use millennium_utils::config::{AppUrl, Config, PatternKind, WindowUrl};
#[cfg(feature = "shell-scope")]
use millennium_utils::config::{ShellAllowedArg, ShellAllowedArgs, ShellAllowlistScope};
use millennium_utils::html::{inject_nonce_token, parse as parse_html, NodeRef};
use proc_macro2::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

use crate::embedded_assets::{AssetOptions, CspHashes, EmbeddedAssets, EmbeddedAssetsError};

/// Necessary data needed by [`context_codegen`] to generate code for a
/// Millennium application context.
pub struct ContextData {
	pub dev: bool,
	pub config: Config,
	pub config_parent: PathBuf,
	pub root: TokenStream
}

fn load_csp(document: &mut NodeRef, key: &AssetKey, csp_hashes: &mut CspHashes) {
	#[cfg(target_os = "linux")]
	::millennium_utils::html::inject_csp_token(document);
	inject_nonce_token(document);
	if let Ok(inline_script_elements) = document.select("script:not(empty)") {
		let mut scripts = Vec::new();
		for inline_script_el in inline_script_elements {
			let script = inline_script_el.as_node().text_contents();
			let mut hasher = Sha256::new();
			hasher.update(&script);
			let hash = hasher.finalize();
			scripts.push(format!("'sha256-{}'", base64::encode(&hash)));
		}
		csp_hashes.inline_scripts.entry(key.clone().into()).or_default().append(&mut scripts);
	}
}

fn map_core_assets(options: &AssetOptions) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError> {
	#[cfg(any(feature = "isolation", feature = "__isolation-docs"))]
	let pattern = millennium_utils::html::PatternObject::from(&options.pattern);
	let csp = options.csp;
	move |key, path, input, csp_hashes| {
		if path.extension() == Some(OsStr::new("html")) {
			let mut document = parse_html(String::from_utf8_lossy(input).into_owned());

			if csp {
				load_csp(&mut document, key, csp_hashes);

				#[cfg(any(feature = "isolation", feature = "__isolation-docs"))]
				if let millennium_utils::html::PatternObject::Isolation { .. } = &pattern {
					// create the csp for the isolation iframe styling now, to make the runtime less
					// complex
					let mut hasher = Sha256::new();
					hasher.update(millennium_utils::pattern::isolation::IFRAME_STYLE);
					let hash = hasher.finalize();
					csp_hashes.styles.push(format!("'sha256-{}'", base64::encode(&hash)));
				}
			}

			*input = document.to_string().as_bytes().to_vec();
		}
		Ok(())
	}
}

#[cfg(any(feature = "isolation", feature = "__isolation-docs"))]
fn map_isolation(_options: &AssetOptions, dir: PathBuf) -> impl Fn(&AssetKey, &Path, &mut Vec<u8>, &mut CspHashes) -> Result<(), EmbeddedAssetsError> {
	move |_key, path, input, _csp_hashes| {
		if path.extension() == Some(OsStr::new("html")) {
			let mut isolation_html = millennium_utils::html::parse(String::from_utf8_lossy(input).into_owned());

			// this is appended, so no need to reverse order it
			millennium_utils::html::inject_codegen_isolation_script(&mut isolation_html);

			// temporary workaround for windows not loading assets
			millennium_utils::html::inline_isolation(&mut isolation_html, &dir);

			*input = isolation_html.to_string().as_bytes().to_vec()
		}

		Ok(())
	}
}

/// Build a `millennium::Context` for including in application code.
pub fn context_codegen(data: ContextData) -> Result<TokenStream, EmbeddedAssetsError> {
	let ContextData {
		dev,
		config,
		config_parent,
		root
	} = data;

	let mut options = AssetOptions::new(config.millennium.pattern.clone()).freeze_prototype(config.millennium.security.freeze_prototype);
	let csp = if dev {
		config.millennium.security.dev_csp.clone().or_else(|| config.millennium.security.csp.clone())
	} else {
		config.millennium.security.csp.clone()
	};
	if csp.is_some() {
		options = options.with_csp();
	}

	let app_url = if dev { &config.build.dev_path } else { &config.build.dist_dir };

	let assets = match app_url {
		AppUrl::Url(url) => match url {
			WindowUrl::External(_) => Default::default(),
			WindowUrl::App(path) => {
				if path.components().count() == 0 {
					panic!("The `{}` configuration cannot be empty", if dev { "devPath" } else { "distDir" })
				}
				let assets_path = config_parent.join(path);
				if !assets_path.exists() {
					panic!(
						"The `{}` configuration is set to `{:?}` but this path doesn't exist",
						if dev { "devPath" } else { "distDir" },
						path
					)
				}
				EmbeddedAssets::new(assets_path, map_core_assets(&options))?
			}
			_ => unimplemented!()
		},
		AppUrl::Files(files) => EmbeddedAssets::new(files.iter().map(|p| config_parent.join(p)).collect::<Vec<_>>(), map_core_assets(&options))?,
		_ => unimplemented!()
	};

	#[cfg(any(windows, target_os = "linux"))]
	let out_dir = {
		let out_dir = std::env::var("OUT_DIR")
			.map_err(|_| EmbeddedAssetsError::OutDir)
			.map(PathBuf::from)
			.and_then(|p| p.canonicalize().map_err(|_| EmbeddedAssetsError::OutDir))?;

		// make sure that our output directory is created
		std::fs::create_dir_all(&out_dir).map_err(|_| EmbeddedAssetsError::OutDir)?;

		out_dir
	};

	// handle default window icons for Windows targets
	#[cfg(windows)]
	let default_window_icon = {
		let icon_path = find_icon(&config, &config_parent, |i| i.ends_with(".ico"), "icons/icon.ico");
		ico_icon(&root, &out_dir, icon_path)?
	};
	#[cfg(target_os = "linux")]
	let default_window_icon = {
		let icon_path = find_icon(&config, &config_parent, |i| i.ends_with(".png"), "icons/icon.png");
		png_icon(&root, &out_dir, icon_path)?
	};
	#[cfg(not(any(windows, target_os = "linux")))]
	let default_window_icon = quote!(None);

	let package_name = if let Some(product_name) = &config.package.product_name {
		quote!(#product_name.to_string())
	} else {
		quote!(env!("CARGO_PKG_NAME").to_string())
	};
	let package_version = if let Some(version) = &config.package.version {
		quote!(#version.to_string())
	} else {
		quote!(env!("CARGO_PKG_VERSION").to_string())
	};
	let package_info = quote!(
	  #root::PackageInfo {
		name: #package_name,
		version: #package_version,
		authors: env!("CARGO_PKG_AUTHORS"),
		description: env!("CARGO_PKG_DESCRIPTION"),
	  }
	);

	#[cfg(target_os = "linux")]
	let system_tray_icon = if let Some(tray) = &config.millennium.system_tray {
		let mut system_tray_icon_path = tray.icon_path.clone();
		system_tray_icon_path.set_extension("png");
		if dev {
			let system_tray_icon_path = config_parent.join(system_tray_icon_path).display().to_string();
			quote!(Some(#root::TrayIcon::File(::std::path::PathBuf::from(#system_tray_icon_path))))
		} else {
			let system_tray_icon_file_path = system_tray_icon_path.to_string_lossy().to_string();
			quote!(
			  Some(
				#root::TrayIcon::File(
				  #root::api::path::resolve_path(
					&#config,
					&#package_info,
					&Default::default(),
					#system_tray_icon_file_path,
					Some(#root::api::path::BaseDirectory::Resource)
				  ).expect("failed to resolve resource dir")
				)
			  )
			)
		}
	} else {
		quote!(None)
	};

	#[cfg(not(target_os = "linux"))]
	let system_tray_icon = if let Some(tray) = &config.millennium.system_tray {
		let mut system_tray_icon_path = tray.icon_path.clone();
		system_tray_icon_path.set_extension(if cfg!(windows) { "ico" } else { "png" });
		let system_tray_icon_path = config_parent.join(system_tray_icon_path).display().to_string();
		quote!(Some(#root::TrayIcon::Raw(include_bytes!(#system_tray_icon_path).to_vec())))
	} else {
		quote!(None)
	};

	#[cfg(target_os = "macos")]
	let info_plist = {
		if dev {
			let info_plist_path = config_parent.join("Info.plist");
			if info_plist_path.exists() {
				let info_plist_path = info_plist_path.display().to_string();
				quote!({
					millennium::embed_plist::embed_info_plist!(#info_plist_path);
				})
			} else {
				quote!(())
			}
		} else {
			quote!(())
		}
	};
	#[cfg(not(target_os = "macos"))]
	let info_plist = quote!(());

	let pattern = match &options.pattern {
		PatternKind::Brownfield => quote!(#root::Pattern::Brownfield(std::marker::PhantomData)),
		#[cfg(any(feature = "isolation", feature = "__isolation-docs"))]
		PatternKind::Isolation { dir } => {
			let dir = config_parent.join(dir);
			if !dir.exists() {
				panic!("The isolation dir configuration is set to `{:?}` but this path doesn't exist", dir)
			}

			let key = uuid::Uuid::new_v4().to_string();
			let assets = EmbeddedAssets::new(dir.clone(), map_isolation(&options, dir))?;
			let schema = options.isolation_schema;

			quote!(#root::Pattern::Isolation {
			  assets: ::std::sync::Arc::new(#assets),
			  schema: #schema.into(),
			  key: #key.into(),
			  crypto_keys: std::boxed::Box::new(::millennium::utils::pattern::isolation::Keys::new().expect("unable to generate cryptographically secure keys for Millennium isolation pattern")),
			})
		}
	};

	#[cfg(feature = "shell-scope")]
	let shell_scope_config = {
		use millennium_utils::config::ShellAllowlistOpen;
		use regex::Regex;

		let shell_scopes = get_allowed_clis(&root, &config.millennium.allowlist.shell.scope);

		let shell_scope_open = match &config.millennium.allowlist.shell.open {
			ShellAllowlistOpen::Flag(false) => quote!(::std::option::Option::None),
			ShellAllowlistOpen::Flag(true) => {
				quote!(::std::option::Option::Some(#root::regex::Regex::new("^https?://").unwrap()))
			}
			ShellAllowlistOpen::Validate(regex) => match Regex::new(regex) {
				Ok(_) => quote!(::std::option::Option::Some(#root::regex::Regex::new(#regex).unwrap())),
				Err(error) => {
					let error = error.to_string();
					quote!({
					  compile_error!(#error);
					  ::std::option::Option::Some(#root::regex::Regex::new(#regex).unwrap())
					})
				}
			},
			_ => panic!("unknown shell open format, unable to prepare")
		};

		quote!(#root::ShellScopeConfig {
		  open: #shell_scope_open,
		  scopes: #shell_scopes
		})
	};

	#[cfg(not(feature = "shell-scope"))]
	let shell_scope_config = quote!();

	Ok(quote!(#root::Context::new(
	  #config,
	  ::std::sync::Arc::new(#assets),
	  #default_window_icon,
	  #system_tray_icon,
	  #package_info,
	  #info_plist,
	  #pattern,
	  #shell_scope_config
	)))
}

#[cfg(windows)]
fn ico_icon<P: AsRef<Path>>(root: &TokenStream, out_dir: &Path, path: P) -> Result<TokenStream, EmbeddedAssetsError> {
	use std::fs::File;
	use std::io::Write;

	let path = path.as_ref();
	let bytes = std::fs::read(&path)
		.unwrap_or_else(|_| panic!("failed to read window icon {}", path.display()))
		.to_vec();
	let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes)).unwrap_or_else(|_| panic!("failed to parse window icon {}", path.display()));
	let entry = &icon_dir.entries()[0];
	let rgba = entry
		.decode()
		.unwrap_or_else(|_| panic!("failed to decode window icon {}", path.display()))
		.rgba_data()
		.to_vec();
	let width = entry.width();
	let height = entry.height();

	let out_path = out_dir.join(path.file_name().unwrap());
	let mut out_file = File::create(&out_path).map_err(|error| EmbeddedAssetsError::AssetWrite { path: out_path.clone(), error })?;
	out_file
		.write_all(&rgba)
		.map_err(|error| EmbeddedAssetsError::AssetWrite { path: path.to_owned(), error })?;

	let out_path = out_path.display().to_string();
	let icon = quote!(Some(#root::Icon::Rgba { rgba: include_bytes!(#out_path).to_vec(), width: #width, height: #height }));
	Ok(icon)
}

#[cfg(target_os = "linux")]
fn png_icon<P: AsRef<Path>>(root: &TokenStream, out_dir: &Path, path: P) -> Result<TokenStream, EmbeddedAssetsError> {
	use std::fs::File;
	use std::io::Write;

	let path = path.as_ref();
	let bytes = std::fs::read(&path)
		.unwrap_or_else(|_| panic!("failed to read window icon {}", path.display()))
		.to_vec();
	let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
	let (info, mut reader) = decoder.read_info().unwrap_or_else(|_| panic!("failed to read window icon {}", path.display()));
	let mut buffer: Vec<u8> = Vec::new();
	while let Ok(Some(row)) = reader.next_row() {
		buffer.extend(row);
	}
	let width = info.width;
	let height = info.height;

	let out_path = out_dir.join(path.file_name().unwrap());
	let mut out_file = File::create(&out_path).map_err(|error| EmbeddedAssetsError::AssetWrite { path: out_path.clone(), error })?;
	out_file
		.write_all(&buffer)
		.map_err(|error| EmbeddedAssetsError::AssetWrite { path: path.to_owned(), error })?;

	let out_path = out_path.display().to_string();
	let icon = quote!(Some(#root::Icon::Rgba { rgba: include_bytes!(#out_path).to_vec(), width: #width, height: #height }));
	Ok(icon)
}

#[cfg(any(windows, target_os = "linux"))]
fn find_icon<F: Fn(&&String) -> bool>(config: &Config, config_parent: &Path, predicate: F, default: &str) -> String {
	let icon_path = config
		.millennium
		.bundle
		.icon
		.iter()
		.find(|i| predicate(i))
		.cloned()
		.unwrap_or_else(|| default.to_string());
	config_parent.join(icon_path).display().to_string()
}

#[cfg(feature = "shell-scope")]
fn get_allowed_clis(root: &TokenStream, scope: &ShellAllowlistScope) -> TokenStream {
	let commands = scope
		.0
		.iter()
		.map(|scope| {
			let sidecar = &scope.sidecar;

			let name = &scope.name;
			let name = quote!(#name.into());

			let command = scope.command.to_string_lossy();
			let command = quote!(::std::path::PathBuf::from(#command));

			let args = match &scope.args {
				ShellAllowedArgs::Flag(true) => quote!(::std::option::Option::None),
				ShellAllowedArgs::Flag(false) => quote!(::std::option::Option::Some(::std::vec![])),
				ShellAllowedArgs::List(list) => {
					let list = list.iter().map(|arg| match arg {
						ShellAllowedArg::Fixed(fixed) => {
							quote!(#root::scope::ShellScopeAllowedArg::Fixed(#fixed.into()))
						}
						ShellAllowedArg::Var { validator } => {
							let validator = match regex::Regex::new(validator) {
								Ok(regex) => {
									let regex = regex.as_str();
									quote!(#root::regex::Regex::new(#regex).unwrap())
								}
								Err(error) => {
									let error = error.to_string();
									quote!({
									  compile_error!(#error);
									  #root::regex::Regex::new(#validator).unwrap()
									})
								}
							};

							quote!(#root::scope::ShellScopeAllowedArg::Var { validator: #validator })
						}
						_ => panic!("unknown shell scope arg, unable to prepare")
					});

					quote!(::std::option::Option::Some(::std::vec![#(#list),*]))
				}
				_ => panic!("unknown shell scope command, unable to prepare")
			};

			(
				quote!(#name),
				quote!(
				  #root::scope::ShellScopeAllowedCommand {
					command: #command,
					args: #args,
					sidecar: #sidecar,
				  }
				)
			)
		})
		.collect::<Vec<_>>();

	if commands.is_empty() {
		quote!(::std::collections::HashMap::new())
	} else {
		let insertions = commands.iter().map(|(name, value)| quote!(hashmap.insert(#name, #value);));

		quote!({
		  let mut hashmap = ::std::collections::HashMap::new();
		  #(#insertions)*
		  hashmap
		})
	}
}
