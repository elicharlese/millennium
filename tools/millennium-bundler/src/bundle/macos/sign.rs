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

use std::{ffi::OsString, fs::File, io::prelude::*, path::PathBuf, process::Command};

use anyhow::Context;
use log::info;
use regex::Regex;

use crate::{bundle::common::CommandExt, Settings};

const KEYCHAIN_ID: &str = "millennium-build.keychain";
const KEYCHAIN_PWD: &str = "millennium-build";

// Import certificate from ENV variables.
// APPLE_CERTIFICATE is the p12 certificate base64 encoded.
// By example you can use; openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt
// Then use the value of the base64 in APPLE_CERTIFICATE env variable.
// You need to set APPLE_CERTIFICATE_PASSWORD to the password you set when youy exported your certificate.
// https://help.apple.com/xcode/mac/current/#/dev154b28f09 see: `Export a signing certificate`
pub fn setup_keychain(certificate_encoded: OsString, certificate_password: OsString) -> crate::Result<()> {
	// we delete any previous version of the keychain if present
	delete_keychain();
	info!("Setting up keychain from environment variables");

	let keychain_list_output = Command::new("security").args(["list-keychain", "-d", "user"]).output()?;

	let tmp_dir = tempfile::tempdir()?;
	let cert_path = tmp_dir.path().join("cert.p12").to_string_lossy().to_string();
	let cert_path_tmp = tmp_dir.path().join("cert.p12.tmp").to_string_lossy().to_string();

	let certificate_encoded = certificate_encoded
		.to_str()
		.expect("failed to convert APPLE_CERTIFICATE to string")
		.as_bytes();
	let certificate_password = certificate_password
		.to_str()
		.expect("failed to convert APPLE_CERTIFICATE_PASSWORD to string")
		.to_string();

	// Because the certificate may contain whitespace, decoding may be broken: https://github.com/marshallpierce/rust-base64/issues/105
	// We'll use the builtin base64 command from the OS
	let mut tmp_cert = File::create(cert_path_tmp.clone())?;
	tmp_cert.write_all(certificate_encoded)?;

	Command::new("base64")
		.args(["--decode", "-i", &cert_path_tmp, "-o", &cert_path])
		.output_ok()
		.context("failed to decode certificate")?;

	Command::new("security")
		.args(["create-keychain", "-p", KEYCHAIN_PWD, KEYCHAIN_ID])
		.output_ok()
		.context("failed to create keychain")?;

	Command::new("security")
		.args(["unlock-keychain", "-p", KEYCHAIN_PWD, KEYCHAIN_ID])
		.output_ok()
		.context("failed to unlock keychain")?;

	Command::new("security")
		.args([
			"import",
			&cert_path,
			"-k",
			KEYCHAIN_ID,
			"-P",
			&certificate_password,
			"-T",
			"/usr/bin/codesign",
			"-T",
			"/usr/bin/pkgbuild",
			"-T",
			"/usr/bin/productbuild"
		])
		.output_ok()
		.context("failed to import keychain certificate")?;

	Command::new("security")
		.args(vec!["set-keychain-settings", "-t", "3600", "-u", KEYCHAIN_ID])
		.output_ok()
		.context("failed to set keychain settings")?;

	Command::new("security")
		.args(vec!["set-key-partition-list", "-S", "apple-tool:,apple:,codesign:", "-s", "-k", KEYCHAIN_PWD, KEYCHAIN_ID])
		.output_ok()
		.context("failed to set keychain settings")?;

	let current_keychains = String::from_utf8_lossy(&keychain_list_output.stdout)
		.split('\n')
		.map(|line| line.trim_matches(|c: char| c.is_whitespace() || c == '"').to_string())
		.filter(|l| !l.is_empty())
		.collect::<Vec<String>>();

	Command::new("security")
		.args(["list-keychain", "-d", "user", "-s"])
		.args(current_keychains)
		.arg(KEYCHAIN_ID)
		.output_ok()
		.context("failed to list keychain")?;

	Ok(())
}

pub fn delete_keychain() {
	// delete keychain if needed and skip any error
	let _ = Command::new("security").arg("delete-keychain").arg(KEYCHAIN_ID).output_ok();
}

pub fn sign(path_to_sign: PathBuf, identity: &str, settings: &Settings, is_an_executable: bool) -> crate::Result<()> {
	info!(action = "Signing"; "{} with identity \"{}\"", path_to_sign.display(), identity);

	let setup_keychain = if let (Some(certificate_encoded), Some(certificate_password)) =
		(std::env::var_os("APPLE_CERTIFICATE"), std::env::var_os("APPLE_CERTIFICATE_PASSWORD"))
	{
		setup_keychain(certificate_encoded, certificate_password)?;
		true
	} else {
		false
	};

	let res = try_sign(path_to_sign, identity, settings, is_an_executable, setup_keychain);

	if setup_keychain {
		// delete the keychain after signing
		delete_keychain();
	}

	res
}

fn try_sign(path_to_sign: PathBuf, identity: &str, settings: &Settings, is_an_executable: bool, millennium_keychain: bool) -> crate::Result<()> {
	let mut args = vec!["--force", "-s", identity];

	if millennium_keychain {
		args.push("--keychain");
		args.push(KEYCHAIN_ID);
	}

	if let Some(entitlements_path) = &settings.macos().entitlements {
		info!("using entitlements file at {}", entitlements_path);
		args.push("--entitlements");
		args.push(entitlements_path);
	}

	if is_an_executable {
		args.push("--options");
		args.push("runtime");
	}

	if path_to_sign.is_dir() {
		args.push("--deep");
	}

	Command::new("codesign")
		.args(args)
		.arg(path_to_sign.to_string_lossy().to_string())
		.output_ok()
		.context("failed to sign app")?;

	Ok(())
}

pub fn notarize(app_bundle_path: PathBuf, auth_args: Vec<String>, settings: &Settings) -> crate::Result<()> {
	let identifier = settings.bundle_identifier();

	let bundle_stem = app_bundle_path.file_stem().expect("failed to get bundle filename");

	let tmp_dir = tempfile::tempdir()?;
	let zip_path = tmp_dir.path().join(format!("{}.zip", bundle_stem.to_string_lossy()));
	let zip_args = vec![
		"-c",
		"-k",
		"--keepParent",
		"--sequesterRsrc",
		app_bundle_path.to_str().expect("failed to convert bundle_path to string"),
		zip_path.to_str().expect("failed to convert zip_path to string"),
	];

	// use ditto to create a PKZip almost identical to Finder
	// this remove almost 99% of false alarm in notarization
	Command::new("ditto").args(zip_args).output_ok().context("failed to zip app with ditto")?;

	// sign the zip file
	if let Some(identity) = &settings.macos().signing_identity {
		sign(zip_path.clone(), identity, settings, false)?;
	};

	let mut notarize_args = vec![
		"altool",
		"--notarize-app",
		"-f",
		zip_path.to_str().expect("failed to convert zip_path to string"),
		"--primary-bundle-id",
		identifier,
	];

	if let Some(provider_short_name) = &settings.macos().provider_short_name {
		notarize_args.push("--asc-provider");
		notarize_args.push(provider_short_name);
	}

	info!(action = "Notarizing"; "{}", app_bundle_path.display());

	let output = Command::new("xcrun")
		.args(notarize_args)
		.args(auth_args.clone())
		.output_ok()
		.context("failed to upload app to Apple notarization servers")?;

	if !output.status.success() {
		return Err(anyhow::anyhow!(format!("failed to upload app to Apple's notarization servers. {}", std::str::from_utf8(&output.stdout)?)).into());
	}

	let mut notarize_response = std::str::from_utf8(&output.stdout)?.to_string();
	notarize_response.push('\n');
	notarize_response.push_str(std::str::from_utf8(&output.stderr)?);
	notarize_response.push('\n');
	if let Some(uuid) = Regex::new(r"\nRequestUUID = (.+?)\n")?.captures_iter(&notarize_response).next() {
		info!("notarization started; waiting for Apple response...");
		let uuid = uuid[1].to_string();
		get_notarization_status(uuid, auth_args, settings)?;
		staple_app(app_bundle_path.clone())?;
	} else {
		return Err(anyhow::anyhow!("failed to parse RequestUUID from upload output. {}", notarize_response).into());
	}

	Ok(())
}

fn staple_app(mut app_bundle_path: PathBuf) -> crate::Result<()> {
	let app_bundle_path_clone = app_bundle_path.clone();
	let filename = app_bundle_path_clone
		.file_name()
		.expect("failed to get bundle filename")
		.to_str()
		.expect("failed to convert bundle filename to string");

	app_bundle_path.pop();

	Command::new("xcrun")
		.args(vec!["stapler", "staple", "-v", filename])
		.current_dir(app_bundle_path)
		.output_ok()
		.context("failed to staple app")?;

	Ok(())
}

fn get_notarization_status(uuid: String, auth_args: Vec<String>, settings: &Settings) -> crate::Result<()> {
	std::thread::sleep(std::time::Duration::from_secs(10));
	let result = Command::new("xcrun")
		.args(vec!["altool", "--notarization-info", &uuid])
		.args(auth_args.clone())
		.output_ok();

	if let Ok(output) = result {
		let mut notarize_status = std::str::from_utf8(&output.stdout)?.to_string();
		notarize_status.push('\n');
		notarize_status.push_str(std::str::from_utf8(&output.stderr)?);
		notarize_status.push('\n');
		if let Some(status) = Regex::new(r"\n *Status: (.+?)\n")?.captures_iter(&notarize_status).next() {
			let status = status[1].to_string();
			if status == "in progress" {
				get_notarization_status(uuid, auth_args, settings)
			} else if status == "invalid" {
				Err(anyhow::anyhow!(format!("Apple failed to notarize your app. {}", notarize_status)).into())
			} else if status != "success" {
				Err(anyhow::anyhow!(format!("Unknown notarize status {}. {}", status, notarize_status)).into())
			} else {
				Ok(())
			}
		} else {
			get_notarization_status(uuid, auth_args, settings)
		}
	} else {
		get_notarization_status(uuid, auth_args, settings)
	}
}

pub fn notarize_auth_args() -> crate::Result<Vec<String>> {
	match (std::env::var_os("APPLE_ID"), std::env::var_os("APPLE_PASSWORD")) {
		(Some(apple_id), Some(apple_password)) => {
			let apple_id = apple_id.to_str().expect("failed to convert APPLE_ID to string").to_string();
			let apple_password = apple_password.to_str().expect("failed to convert APPLE_PASSWORD to string").to_string();
			Ok(vec!["-u".to_string(), apple_id, "-p".to_string(), apple_password])
		}
		_ => match (std::env::var_os("APPLE_API_KEY"), std::env::var_os("APPLE_API_ISSUER")) {
			(Some(api_key), Some(api_issuer)) => {
				let api_key = api_key.to_str().expect("failed to convert APPLE_API_KEY to string").to_string();
				let api_issuer = api_issuer.to_str().expect("failed to convert APPLE_API_ISSUER to string").to_string();
				Ok(vec!["--apiKey".to_string(), api_key, "--apiIssuer".to_string(), api_issuer])
			}
			_ => Err(anyhow::anyhow!("no APPLE_ID & APPLE_PASSWORD or APPLE_API_KEY & APPLE_API_ISSUER environment variables found").into())
		}
	}
}
