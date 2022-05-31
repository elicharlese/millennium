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
	collections::{BTreeMap, HashMap},
	fs::{create_dir_all, read_to_string, remove_dir_all, rename, write, File},
	io::{Cursor, Read, Write},
	path::{Path, PathBuf},
	process::Command
};

use anyhow::Context;
use handlebars::{to_json, Handlebars};
use log::info;
use millennium_utils::resources::resource_relpath;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use uuid::Uuid;
use zip::ZipArchive;

use super::super::sign::{sign, SignParams};
use crate::bundle::{
	common::CommandExt,
	path_utils::{copy_file, FileOpts},
	settings::Settings
};

// URLS for the WIX toolchain.  Can be used for crossplatform compilation.
pub const WIX_URL: &str = "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip";
pub const WIX_SHA256: &str = "2c1888d5d1dba377fc7fa14444cf556963747ff9a0a289a3599cf09da03b9e2e";

// For Cross Platform Complilation.

// const VC_REDIST_X86_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/c8edbb87-c7ec-4500-a461-71e8912d25e9/99ba493d660597490cbb8b3211d2cae4/vc_redist.x86.exe";

// const VC_REDIST_X86_SHA256: &str =
//   "3a43e8a55a3f3e4b73d01872c16d47a19dd825756784f4580187309e7d1fcb74";

// const VC_REDIST_X64_URL: &str =
//     "https://download.visualstudio.microsoft.com/download/pr/9e04d214-5a9d-4515-9960-3d71398d98c3/1e1e62ab57bbb4bf5199e8ce88f040be/vc_redist.x64.exe";

// const VC_REDIST_X64_SHA256: &str =
//   "d6cd2445f68815fe02489fafe0127819e44851e26dfbe702612bc0d223cbbc2b";

// A v4 UUID that was generated specifically for millennium-bundler, to be used as a
// namespace for generating v5 UUIDs from bundle identifier strings.
const UUID_NAMESPACE: [u8; 16] = [0xe4, 0x69, 0xbb, 0x0f, 0xa6, 0x3e, 0x46, 0xff, 0xae, 0x7e, 0xac, 0x50, 0x1e, 0xd2, 0xfd, 0x4c];

/// Mapper between a resource directory name and its ResourceDirectory descriptor.
type ResourceMap = BTreeMap<String, ResourceDirectory>;

#[derive(Debug, Deserialize)]
struct LanguageMetadata {
	#[serde(rename = "asciiCode")]
	ascii_code: usize,
	#[serde(rename = "langId")]
	lang_id: usize
}

/// A binary to bundle with WIX.
/// External binaries or additional project binaries are represented with this data structure.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize)]
struct Binary {
	/// the GUID to use on the WIX XML.
	guid: String,
	/// the id to use on the WIX XML.
	id: String,
	/// the binary path.
	path: String
}

/// A Resource file to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize, Clone)]
struct ResourceFile {
	/// the GUID to use on the WIX XML.
	guid: String,
	/// the id to use on the WIX XML.
	id: String,
	/// the file path.
	path: String
}

/// A resource directory to bundle with WIX.
/// This data structure is needed because WIX requires each path to have its own `id` and `guid`.
#[derive(Serialize)]
struct ResourceDirectory {
	/// the directory path.
	path: String,
	/// the directory name of the described resource.
	name: String,
	/// the files of the described resource directory.
	files: Vec<ResourceFile>,
	/// the directories that are children of the described resource directory.
	directories: Vec<ResourceDirectory>
}

impl ResourceDirectory {
	/// Adds a file to this directory descriptor.
	fn add_file(&mut self, file: ResourceFile) {
		self.files.push(file);
	}

	/// Generates the wix XML string to bundle this directory resources recursively
	fn get_wix_data(self) -> crate::Result<(String, Vec<String>)> {
		let mut files = String::from("");
		let mut file_ids = Vec::new();
		for file in self.files {
			file_ids.push(file.id.clone());
			files.push_str(
				format!(
					r#"<Component Id="{id}" Guid="{guid}" Win64="$(var.Win64)" KeyPath="yes"><File Id="PathFile_{id}" Source="{path}" /></Component>"#,
					id = file.id,
					guid = file.guid,
					path = file.path
				)
				.as_str()
			);
		}
		let mut directories = String::from("");
		for directory in self.directories {
			let (wix_string, ids) = directory.get_wix_data()?;
			for id in ids {
				file_ids.push(id)
			}
			directories.push_str(wix_string.as_str());
		}
		let wix_string = if self.name.is_empty() {
			format!("{}{}", files, directories)
		} else {
			format!(
				r#"<Directory Id="I{id}" Name="{name}">{files}{directories}</Directory>"#,
				id = Uuid::new_v4().as_simple(),
				name = self.name,
				files = files,
				directories = directories,
			)
		};

		Ok((wix_string, file_ids))
	}
}

/// Copies the icon to the binary path, under the `resources` folder,
/// and returns the path to the file.
fn copy_icon(settings: &Settings, filename: &str, path: &Path) -> crate::Result<PathBuf> {
	let base_dir = settings.project_out_directory();

	let resource_dir = base_dir.join("resources");
	std::fs::create_dir_all(&resource_dir)?;
	let icon_target_path = resource_dir.join(filename);

	let icon_path = std::env::current_dir()?.join(&path);

	copy_file(
		icon_path,
		&icon_target_path,
		&FileOpts {
			overwrite: true,
			..Default::default()
		}
	)?;

	Ok(icon_target_path)
}

/// Function used to download Wix and VC_REDIST. Checks SHA256 to verify the download.
fn download_and_verify(url: &str, hash: &str) -> crate::Result<Vec<u8>> {
	info!(action = "Downloading"; "{}", url);

	let response = attohttpc::get(url).send()?;

	let data: Vec<u8> = response.bytes()?;

	info!("Validating hash");

	let mut hasher = sha2::Sha256::new();
	hasher.update(&data);

	let url_hash = hasher.finalize().to_vec();
	let expected_hash = hex::decode(hash)?;

	if expected_hash == url_hash { Ok(data) } else { Err(crate::Error::HashError) }
}

/// The app installer output path.
fn app_installer_output_path(settings: &Settings, language: &str) -> crate::Result<PathBuf> {
	let arch = match settings.binary_arch() {
		"x86" => "x86",
		"x86_64" => "x64",
		target => return Err(crate::Error::ArchError(format!("Unsupported architecture: {}", target)))
	};

	let package_base_name = format!("{}_{}_{}_{}", settings.main_binary_name().replace(".exe", ""), settings.version_string(), arch, language,);

	Ok(settings
		.project_out_directory()
		.to_path_buf()
		.join(format!("bundle/msi/{}.msi", package_base_name)))
}

/// Extracts the zips from Wix and VC_REDIST into a useable path.
fn extract_zip(data: &[u8], path: &Path) -> crate::Result<()> {
	let cursor = Cursor::new(data);

	let mut zipa = ZipArchive::new(cursor)?;

	for i in 0..zipa.len() {
		let mut file = zipa.by_index(i)?;
		let dest_path = path.join(file.name());
		let parent = dest_path.parent().expect("Failed to get parent");

		if !parent.exists() {
			create_dir_all(parent)?;
		}

		let mut buff: Vec<u8> = Vec::new();
		file.read_to_end(&mut buff)?;
		let mut fileout = File::create(dest_path).expect("Failed to open file");

		fileout.write_all(&buff)?;
	}

	Ok(())
}

/// Generates the UUID for the Wix template.
fn generate_package_guid(settings: &Settings) -> Uuid {
	generate_guid(settings.bundle_identifier().as_bytes())
}

/// Generates a GUID.
fn generate_guid(key: &[u8]) -> Uuid {
	let namespace = Uuid::from_bytes(UUID_NAMESPACE);
	Uuid::new_v5(&namespace, key)
}

// Specifically goes and gets Wix and verifies the download via Sha256
pub fn get_and_extract_wix(path: &Path) -> crate::Result<()> {
	info!("Verifying WiX package");

	let data = download_and_verify(WIX_URL, WIX_SHA256)?;

	info!("Extracting WiX");

	extract_zip(&data, path)
}

/// Runs the Candle.exe executable for Wix. Candle parses the wxs file and generates the code for building the
/// installer.
fn run_candle(settings: &Settings, wix_toolset_path: &Path, cwd: &Path, wxs_file_path: &Path) -> crate::Result<()> {
	let arch = match settings.binary_arch() {
		"x86_64" => "x64",
		"x86" => "x86",
		target => return Err(crate::Error::ArchError(format!("unsupported target: {}", target)))
	};

	let main_binary = settings
		.binaries()
		.iter()
		.find(|bin| bin.main())
		.ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;

	let args = vec![
		"-arch".to_string(),
		arch.to_string(),
		wxs_file_path.to_string_lossy().to_string(),
		format!("-dSourceDir={}", settings.binary_path(main_binary).display()),
	];

	let candle_exe = wix_toolset_path.join("candle.exe");

	info!(action = "Running"; "candle for {:?}", wxs_file_path);

	Command::new(&candle_exe)
		.args(&args)
		.current_dir(cwd)
		.output_ok()
		.context("error running candle")?;

	Ok(())
}

/// Runs the Light.exe file. Light takes the generated code from Candle and produces an MSI Installer.
fn run_light(wix_toolset_path: &Path, build_path: &Path, arguments: Vec<String>, output_path: &Path) -> crate::Result<()> {
	let light_exe = wix_toolset_path.join("light.exe");

	let mut args: Vec<String> = vec![
		"-ext".to_string(),
		"WixUIExtension".to_string(),
		"-ext".to_string(),
		"WixUtilExtension".to_string(),
		"-o".to_string(),
		output_path.display().to_string(),
	];

	for p in arguments {
		args.push(p);
	}

	Command::new(&light_exe)
		.args(&args)
		.current_dir(build_path)
		.output_ok()
		.context("error running light")?;

	Ok(())
}

// fn get_icon_data() -> crate::Result<()> {
//   Ok(())
// }

// Entry point for bundling and creating the MSI installer. For now the only supported platform is Windows x64.
pub fn build_wix_app_installer(settings: &Settings, wix_toolset_path: &Path) -> crate::Result<Vec<PathBuf>> {
	let arch = match settings.binary_arch() {
		"x86_64" => "x64",
		"x86" => "x86",
		target => return Err(crate::Error::ArchError(format!("unsupported target: {}", target)))
	};

	// target only supports x64.
	info!("Target: {}", arch);

	let main_binary = settings
		.binaries()
		.iter()
		.find(|bin| bin.main())
		.ok_or_else(|| anyhow::anyhow!("Failed to get main binary"))?;
	let app_exe_source = settings.binary_path(main_binary);
	let try_sign = |file_path: &PathBuf| -> crate::Result<()> {
		if let Some(certificate_thumbprint) = &settings.windows().certificate_thumbprint {
			info!(action = "Signing"; "{}", file_path.display());
			sign(
				&file_path,
				&SignParams {
					digest_algorithm: settings
						.windows()
						.digest_algorithm
						.as_ref()
						.map(|algorithm| algorithm.to_string())
						.unwrap_or_else(|| "sha256".to_string()),
					certificate_thumbprint: certificate_thumbprint.to_string(),
					timestamp_url: settings.windows().timestamp_url.as_ref().map(|url| url.to_string()),
					tsp: settings.windows().tsp
				}
			)?;
		}
		Ok(())
	};

	try_sign(&app_exe_source)?;

	// ensure that `target/{release, debug}/wix` folder exists
	std::fs::create_dir_all(settings.project_out_directory().join("wix"))?;

	let output_path = settings.project_out_directory().join("wix").join(arch);

	let mut data = BTreeMap::new();

	let language_map: HashMap<String, LanguageMetadata> = serde_json::from_str(include_str!("./languages.json")).unwrap();

	if let Some(wix) = &settings.windows().wix {
		if let Some(license) = &wix.license {
			if license.ends_with(".rtf") {
				data.insert("license", to_json(license));
			} else {
				let license_contents = read_to_string(&license)?;
				let license_rtf = format!(
					r#"{{\rtf1\ansi\ansicpg1252\deff0\nouicompat\deflang1033{{\fonttbl{{\f0\fnil\fcharset0 Calibri;}}}}
{{\*\generator Riched20 10.0.18362}}\viewkind4\uc1
\pard\sa200\sl276\slmult1\f0\fs22\lang9 {}\par
}}
 "#,
					license_contents.replace('\n', "\\par ")
				);
				let rtf_output_path = settings.project_out_directory().join("wix").join("LICENSE.rtf");
				std::fs::write(&rtf_output_path, license_rtf)?;
				data.insert("license", to_json(rtf_output_path));
			}
		}
	}

	let configured_languages = settings.windows().wix.as_ref().map(|w| w.language.clone()).unwrap_or_default();

	data.insert("product_name", to_json(settings.product_name()));
	data.insert("version", to_json(settings.version_string()));
	let bundle_id = settings.bundle_identifier();
	let manufacturer = bundle_id.split('.').nth(1).unwrap_or(bundle_id);
	data.insert("bundle_id", to_json(bundle_id));
	data.insert("manufacturer", to_json(manufacturer));
	let upgrade_code = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("{}.app.x64", &settings.main_binary_name()).as_bytes()).to_string();

	data.insert("upgrade_code", to_json(&upgrade_code.as_str()));
	data.insert("allow_downgrades", to_json(settings.windows().allow_downgrades));

	let path_guid = generate_package_guid(settings).to_string();
	data.insert("path_component_guid", to_json(&path_guid.as_str()));

	let shortcut_guid = generate_package_guid(settings).to_string();
	data.insert("shortcut_guid", to_json(&shortcut_guid.as_str()));

	let app_exe_name = settings.main_binary_name().to_string();
	data.insert("app_exe_name", to_json(&app_exe_name));

	let binaries = generate_binaries_data(settings)?;

	let binaries_json = to_json(&binaries);
	data.insert("binaries", binaries_json);

	let resources = generate_resource_data(settings)?;
	let mut resources_wix_string = String::from("");
	let mut files_ids = Vec::new();
	for (_, dir) in resources {
		let (wix_string, ids) = dir.get_wix_data()?;
		resources_wix_string.push_str(wix_string.as_str());
		for id in ids {
			files_ids.push(id);
		}
	}

	data.insert("resources", to_json(resources_wix_string));
	data.insert("resource_file_ids", to_json(files_ids));

	let merge_modules = get_merge_modules(settings)?;
	data.insert("merge_modules", to_json(merge_modules));

	data.insert("app_exe_source", to_json(&app_exe_source));

	// copy icon from `settings.windows().icon_path` folder to resource folder near msi
	let icon_path = copy_icon(settings, "icon.ico", &settings.windows().icon_path)?;

	data.insert("icon_path", to_json(icon_path));

	let mut fragment_paths = Vec::new();
	let mut handlebars = Handlebars::new();
	let mut has_custom_template = false;
	let mut install_webview = settings.windows().webview_fixed_runtime_path.is_none();
	let mut enable_elevated_update_task = false;

	if let Some(wix) = &settings.windows().wix {
		data.insert("component_group_refs", to_json(&wix.component_group_refs));
		data.insert("component_refs", to_json(&wix.component_refs));
		data.insert("feature_group_refs", to_json(&wix.feature_group_refs));
		data.insert("feature_refs", to_json(&wix.feature_refs));
		data.insert("merge_refs", to_json(&wix.merge_refs));
		fragment_paths = wix.fragment_paths.clone();
		if wix.skip_webview_install {
			install_webview = false;
		}
		enable_elevated_update_task = wix.enable_elevated_update_task;

		if let Some(temp_path) = &wix.template {
			let template = read_to_string(temp_path)?;
			handlebars
				.register_template_string("main.wxs", &template)
				.map_err(|e| e.to_string())
				.expect("Failed to setup custom handlebar template");
			has_custom_template = true;
		}

		if let Some(banner_path) = &wix.banner_path {
			let filename = banner_path.file_name().unwrap().to_string_lossy().into_owned();
			data.insert("banner_path", to_json(copy_icon(settings, &filename, banner_path)?));
		}

		if let Some(dialog_image_path) = &wix.dialog_image_path {
			let filename = dialog_image_path.file_name().unwrap().to_string_lossy().into_owned();
			data.insert("dialog_image_path", to_json(copy_icon(settings, &filename, dialog_image_path)?));
		}
	}

	if !has_custom_template {
		handlebars
			.register_template_string("main.wxs", include_str!("../templates/main.wxs"))
			.map_err(|e| e.to_string())
			.expect("Failed to setup handlebar template");
	}

	if install_webview {
		data.insert("install_webview", to_json(true));
	}

	if output_path.exists() {
		remove_dir_all(&output_path)?;
	}

	create_dir_all(&output_path)?;

	if enable_elevated_update_task {
		data.insert(
			"msiexec_args",
			to_json(
				settings
					.updater()
					.and_then(|updater| updater.msiexec_args.clone())
					.map(|args| args.join(" "))
					.unwrap_or_else(|| "/passive".to_string())
			)
		);
		// Create the update task XML
		let mut skip_uac_task = Handlebars::new();
		let xml = include_str!("../templates/update-task.xml");
		skip_uac_task
			.register_template_string("update.xml", xml)
			.map_err(|e| e.to_string())
			.expect("Failed to setup Update Task handlebars");
		let temp_xml_path = output_path.join("update.xml");
		let update_content = skip_uac_task.render("update.xml", &data)?;
		write(&temp_xml_path, update_content)?;

		// Create the Powershell script to install the task
		let mut skip_uac_task_installer = Handlebars::new();
		let xml = include_str!("../templates/install-task.ps1");
		skip_uac_task_installer
			.register_template_string("install-task.ps1", xml)
			.map_err(|e| e.to_string())
			.expect("Failed to setup Update Task Installer handlebars");
		let temp_ps1_path = output_path.join("install-task.ps1");
		let install_script_content = skip_uac_task_installer.render("install-task.ps1", &data)?;
		write(&temp_ps1_path, install_script_content)?;

		// Create the Powershell script to uninstall the task
		let mut skip_uac_task_uninstaller = Handlebars::new();
		let xml = include_str!("../templates/uninstall-task.ps1");
		skip_uac_task_uninstaller
			.register_template_string("uninstall-task.ps1", xml)
			.map_err(|e| e.to_string())
			.expect("Failed to setup Update Task Uninstaller handlebars");
		let temp_ps1_path = output_path.join("uninstall-task.ps1");
		let install_script_content = skip_uac_task_uninstaller.render("uninstall-task.ps1", &data)?;
		write(&temp_ps1_path, install_script_content)?;

		data.insert("enable_elevated_update_task", to_json(true));
	}

	let main_wxs_path = output_path.join("main.wxs");
	write(&main_wxs_path, handlebars.render("main.wxs", &data)?)?;

	let mut candle_inputs = vec!["main.wxs".into()];

	let current_dir = std::env::current_dir()?;
	for fragment_path in fragment_paths {
		candle_inputs.push(current_dir.join(fragment_path));
	}

	for wxs in &candle_inputs {
		run_candle(settings, wix_toolset_path, &output_path, wxs)?;
	}

	let mut output_paths = Vec::new();

	for (language, language_config) in configured_languages.0 {
		let language_metadata = language_map.get(&language).unwrap_or_else(|| {
			panic!("Language {} not found. It must be one of {}", language, language_map.keys().cloned().collect::<Vec<String>>().join(", "))
		});

		let locale_contents = match language_config.locale_path {
			Some(p) => read_to_string(p)?,
			None => format!(
				r#"<WixLocalization Culture="{}" xmlns="http://schemas.microsoft.com/wix/2006/localization"></WixLocalization>"#,
				language.to_lowercase(),
			)
		};

		let locale_strings = include_str!("./default-locale-strings.xml")
			.replace("__language__", &language_metadata.lang_id.to_string())
			.replace("__codepage__", &language_metadata.ascii_code.to_string())
			.replace("__productName__", settings.product_name());

		let mut unset_locale_strings = String::new();
		let prefix_len = "<String ".len();
		for locale_string in locale_strings.split('\n').filter(|s| !s.is_empty()) {
			// strip `<String ` prefix and `>{value}</String` suffix.
			let id = locale_string
				.chars()
				.skip(prefix_len)
				.take(locale_string.find('>').unwrap() - prefix_len)
				.collect::<String>();
			if !locale_contents.contains(&id) {
				unset_locale_strings.push_str(locale_string);
			}
		}

		let locale_contents = locale_contents.replace("</WixLocalization>", &format!("{}</WixLocalization>", unset_locale_strings));
		let locale_path = output_path.join("locale.wxl");
		{
			let mut fileout = File::create(&locale_path).expect("Failed to create locale file");
			fileout.write_all(locale_contents.as_bytes())?;
		}

		let arguments = vec![
			format!(
				"-cultures:{}",
				if language == "en-US" {
					language.to_lowercase()
				} else {
					format!("{};en-US", language.to_lowercase())
				}
			),
			"-loc".into(),
			locale_path.display().to_string(),
			"*.wixobj".into(),
		];
		let msi_output_path = output_path.join("output.msi");
		let msi_path = app_installer_output_path(settings, &language)?;
		create_dir_all(msi_path.parent().unwrap())?;

		info!(action = "Running"; "light for {}", msi_path.display());

		run_light(wix_toolset_path, &output_path, arguments, &msi_output_path)?;
		rename(&msi_output_path, &msi_path)?;
		try_sign(&msi_path)?;
		output_paths.push(msi_path);
	}

	Ok(output_paths)
}

/// Generates the data required for the external binaries and extra binaries bundling.
fn generate_binaries_data(settings: &Settings) -> crate::Result<Vec<Binary>> {
	let mut binaries = Vec::new();
	let cwd = std::env::current_dir()?;
	let tmp_dir = std::env::temp_dir();
	for src in settings.external_binaries() {
		let src = src?;
		let binary_path = cwd.join(&src);
		let dest_filename = src
			.file_name()
			.expect("failed to extract external binary filename")
			.to_string_lossy()
			.replace(&format!("-{}", settings.target()), "");
		let dest = tmp_dir.join(dest_filename);
		std::fs::copy(binary_path, &dest)?;

		binaries.push(Binary {
			guid: Uuid::new_v4().to_string(),
			path: dest.into_os_string().into_string().expect("failed to read external binary path"),
			id: format!("I{}", Uuid::new_v4().as_simple())
		});
	}

	for bin in settings.binaries() {
		if !bin.main() {
			binaries.push(Binary {
				guid: Uuid::new_v4().to_string(),
				path: settings
					.binary_path(bin)
					.into_os_string()
					.into_string()
					.expect("failed to read binary path"),
				id: format!("I{}", Uuid::new_v4().as_simple())
			})
		}
	}

	Ok(binaries)
}

#[derive(Serialize)]
struct MergeModule {
	name: String,
	path: String
}

fn get_merge_modules(settings: &Settings) -> crate::Result<Vec<MergeModule>> {
	let mut merge_modules = Vec::new();
	let regex = Regex::new(r"[^\w\d\.]")?;
	for msm in glob::glob(settings.project_out_directory().join("*.msm").to_string_lossy().to_string().as_str())? {
		let path = msm?;
		let filename = path
			.file_name()
			.expect("failed to extract merge module filename")
			.to_os_string()
			.into_string()
			.expect("failed to convert merge module filename to string");
		merge_modules.push(MergeModule {
			name: regex.replace_all(&filename, "").to_string(),
			path: path.to_string_lossy().to_string()
		});
	}
	Ok(merge_modules)
}

/// Generates the data required for the resource bundling on wix
fn generate_resource_data(settings: &Settings) -> crate::Result<ResourceMap> {
	let mut resources = ResourceMap::new();
	let cwd = std::env::current_dir()?;

	let mut added_resources = Vec::new();

	for src in settings.resource_files() {
		let src = src?;

		let resource_path = cwd
			.join(src.clone())
			.into_os_string()
			.into_string()
			.expect("failed to read resource path");

		// In some glob resource paths like `assets/**/*` a file might appear twice
		// because the `millennium_utils::resources::ResourcePaths` iterator also reads a directory
		// when it finds one, so we must check it before processing the file.
		if added_resources.contains(&resource_path) {
			continue;
		}

		added_resources.push(resource_path.clone());

		let resource_entry = ResourceFile {
			id: format!("I{}", Uuid::new_v4().as_simple()),
			guid: Uuid::new_v4().to_string(),
			path: resource_path
		};

		// split the resource path directories
		let target_path = resource_relpath(&src);
		let components_count = target_path.components().count();
		let directories = target_path
			.components()
			.take(components_count - 1) // the last component is the file
			.collect::<Vec<_>>();

		// transform the directory structure to a chained vec structure
		let first_directory = directories
			.first()
			.map(|d| d.as_os_str().to_string_lossy().into_owned())
			.unwrap_or_else(String::new);

		if !resources.contains_key(&first_directory) {
			resources.insert(
				first_directory.clone(),
				ResourceDirectory {
					path: first_directory.clone(),
					name: first_directory.clone(),
					directories: vec![],
					files: vec![]
				}
			);
		}

		let mut directory_entry = resources.get_mut(&first_directory).expect("Unable to handle resources");

		let mut path = String::new();
		// the first component is already parsed on `first_directory` so we skip(1)
		for directory in directories.into_iter().skip(1) {
			let directory_name = directory
				.as_os_str()
				.to_os_string()
				.into_string()
				.expect("failed to read resource folder name");
			path.push_str(directory_name.as_str());
			path.push(std::path::MAIN_SEPARATOR);

			let index = directory_entry.directories.iter().position(|f| f.path == path);
			match index {
				Some(i) => directory_entry = directory_entry.directories.get_mut(i).unwrap(),
				None => {
					directory_entry.directories.push(ResourceDirectory {
						path: path.clone(),
						name: directory_name,
						directories: vec![],
						files: vec![]
					});
					directory_entry = directory_entry.directories.iter_mut().last().unwrap();
				}
			}
		}
		directory_entry.add_file(resource_entry);
	}

	let mut dlls = Vec::new();

	let out_dir = settings.project_out_directory();
	for dll in glob::glob(out_dir.join("*.dll").to_string_lossy().to_string().as_str())? {
		let path = dll?;
		let resource_path = path.to_string_lossy().into_owned();
		let relative_path = path.strip_prefix(&out_dir).unwrap().to_string_lossy().into_owned();
		if !added_resources.iter().any(|r| r.ends_with(&relative_path)) {
			dlls.push(ResourceFile {
				id: format!("I{}", Uuid::new_v4().as_simple()),
				guid: Uuid::new_v4().to_string(),
				path: resource_path
			});
		}
	}

	if !dlls.is_empty() {
		resources.insert(
			"".to_string(),
			ResourceDirectory {
				path: "".to_string(),
				name: "".to_string(),
				directories: vec![],
				files: dlls
			}
		);
	}

	Ok(resources)
}
