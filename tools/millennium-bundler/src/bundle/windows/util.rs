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
	fs::{create_dir_all, File},
	io::{Cursor, Read, Write},
	path::{Path, PathBuf}
};

use anyhow::{bail, Context};
use log::info;
use sha2::Digest;
use zip::ZipArchive;

pub const WEBVIEW2_BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
pub const WEBVIEW2_X86_INSTALLER_GUID: &str = "a17bde80-b5ab-47b5-8bbb-1cbe93fc6ec9";
pub const WEBVIEW2_X64_INSTALLER_GUID: &str = "aa5fd9b3-dc11-4cbc-8343-a50f57b311e1";

use crate::{
	bundle::windows::sign::{sign, SignParams},
	Settings
};

pub fn download(url: &str) -> crate::Result<Vec<u8>> {
	info!(action = "Downloading"; "{}", url);
	let response = attohttpc::get(url).send()?;
	response.bytes().map_err(Into::into)
}

pub enum HashAlgorithm {
	Sha256,
	Sha1
}

/// Function used to download a file and checks SHA256 to verify the download.
pub fn download_and_verify(url: &str, hash: &str, hash_algorithim: HashAlgorithm) -> crate::Result<Vec<u8>> {
	let data = download(url)?;
	info!("validating hash");

	match hash_algorithim {
		HashAlgorithm::Sha256 => {
			let hasher = sha2::Sha256::new();
			verify(&data, hash, hasher)?;
		}
		HashAlgorithm::Sha1 => {
			let hasher = sha1::Sha1::new();
			verify(&data, hash, hasher)?;
		}
	}

	Ok(data)
}

fn verify(data: &Vec<u8>, hash: &str, mut hasher: impl Digest) -> crate::Result<()> {
	hasher.update(data);

	let url_hash = hasher.finalize().to_vec();
	let expected_hash = hex::decode(hash)?;
	if expected_hash == url_hash { Ok(()) } else { Err(crate::Error::HashError) }
}

pub fn validate_version(version: &str) -> anyhow::Result<()> {
	let version = semver::Version::parse(version).context("invalid app version")?;
	if version.major > 255 {
		bail!("app version major number cannot be greater than 255");
	}
	if version.minor > 255 {
		bail!("app version minor number cannot be greater than 255");
	}
	if version.patch > 65535 {
		bail!("app version patch number cannot be greater than 65535");
	}
	if !(version.pre.is_empty() && version.build.is_empty()) {
		bail!("app version cannot have build metadata or pre-release identifier");
	}

	Ok(())
}

pub fn try_sign(file_path: &PathBuf, settings: &Settings) -> crate::Result<()> {
	if let Some(certificate_thumbprint) = settings.windows().certificate_thumbprint.as_ref() {
		info!(action = "Signing"; "{}", file_path.display());
		sign(
			&file_path,
			&SignParams {
				product_name: settings.product_name().into(),
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
}

/// Extracts the zips from memory into a useable path.
pub fn extract_zip(data: &[u8], path: &Path) -> crate::Result<()> {
	let cursor = Cursor::new(data);

	let mut zipa = ZipArchive::new(cursor)?;

	for i in 0..zipa.len() {
		let mut file = zipa.by_index(i)?;

		let dest_path = path.join(file.name());
		if file.is_dir() {
			create_dir_all(&dest_path)?;
			continue;
		}

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

pub fn remove_unc_lossy<P: AsRef<Path>>(p: P) -> PathBuf {
	PathBuf::from(p.as_ref().to_string_lossy().replacen(r"\\?\", "", 1))
}
