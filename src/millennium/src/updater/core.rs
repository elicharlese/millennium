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

#[cfg(feature = "updater")]
#[cfg(not(target_os = "macos"))]
use std::ffi::OsStr;
#[cfg(feature = "updater")]
use std::io::Seek;
use std::{
	collections::HashMap,
	env, fmt,
	io::{Cursor, Read},
	path::{Path, PathBuf},
	str::{from_utf8, FromStr},
	time::Duration
};
#[cfg(target_os = "windows")]
use std::{
	fs::read_dir,
	process::{exit, Command}
};

use base64::decode;
use futures::StreamExt;
use http::{
	header::{HeaderName, HeaderValue},
	HeaderMap, StatusCode
};
use millennium_utils::{platform::current_exe, Env};
use minisign_verify::{PublicKey, Signature};
use semver::Version;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use time::OffsetDateTime;
use url::Url;

use super::error::{Error, Result};
#[cfg(all(feature = "updater", not(target_os = "windows")))]
use crate::api::file::Compression;
#[cfg(feature = "updater")]
use crate::api::file::{ArchiveFormat, Extract, Move};
use crate::{
	api::http::{ClientBuilder, HttpRequestBuilder},
	AppHandle, Manager, Runtime
};

type ShouldInstall = dyn FnOnce(&Version, &RemoteRelease) -> bool + Send;

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RemoteReleaseInner {
	Dynamic(ReleaseManifestPlatform),
	Static { platforms: HashMap<String, ReleaseManifestPlatform> }
}

/// Information about a release returned by the remote update server.
///
/// This type can have one of two shapes: Server format (dynamic format) and static format.
#[derive(Debug)]
pub struct RemoteRelease {
	/// Version to install
	version: Version,
	/// Release notes.
	notes: Option<String>,
	/// Release date.
	pub_date: Option<OffsetDateTime>,
	/// Release data.
	data: RemoteReleaseInner
}

impl<'de> Deserialize<'de> for RemoteRelease {
	fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>
	{
		#[derive(Deserialize)]
		struct InnerRemoteRelease {
			#[serde(alias = "name", deserialize_with = "parse_version")]
			version: Version,
			notes: Option<String>,
			pub_date: Option<String>,
			platforms: Option<HashMap<String, ReleaseManifestPlatform>>,
			// dynamic platform response
			url: Option<Url>,
			signature: Option<String>,
			#[cfg(target_os = "windows")]
			#[serde(default)]
			with_elevated_task: bool
		}

		let release = InnerRemoteRelease::deserialize(deserializer)?;

		let pub_date = if let Some(date) = release.pub_date {
			Some(
				OffsetDateTime::parse(&date, &time::format_description::well_known::Rfc3339)
					.map_err(|e| DeError::custom(format!("invalid value for `pub_date`: {}", e)))?
			)
		} else {
			None
		};

		Ok(RemoteRelease {
			version: release.version,
			notes: release.notes,
			pub_date,
			data: if let Some(platforms) = release.platforms {
				RemoteReleaseInner::Static { platforms }
			} else {
				RemoteReleaseInner::Dynamic(ReleaseManifestPlatform {
					url: release
						.url
						.ok_or_else(|| DeError::custom("the `url` field was not set on the updater response"))?,
					signature: release
						.signature
						.ok_or_else(|| DeError::custom("the `signature` field was not set on the updater response"))?,
					#[cfg(target_os = "windows")]
					with_elevated_task: release.with_elevated_task
				})
			}
		})
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseManifestPlatform {
	/// Download URL for the platform.
	pub url: Url,
	/// Signature for the platform.
	pub signature: String,
	#[cfg(target_os = "windows")]
	#[serde(default)]
	/// On Windows, only attempt to run the updater with elevated privileges.
	pub with_elevated_task: bool
}

fn parse_version<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
where
	D: serde::Deserializer<'de>
{
	let str = String::deserialize(deserializer)?;
	Version::from_str(str.trim_start_matches('v')).map_err(serde::de::Error::custom)
}

impl RemoteRelease {
	/// The semver version of the release.
	pub fn version(&self) -> &Version {
		&self.version
	}

	/// Release notes.
	pub fn notes(&self) -> Option<&String> {
		self.notes.as_ref()
	}

	/// The date the release was published.
	pub fn pub_date(&self) -> Option<&OffsetDateTime> {
		self.pub_date.as_ref()
	}

	/// Get the download URL for this release for the given target.
	pub fn download_url(&self, target: &str) -> Result<&Url> {
		match self.data {
			RemoteReleaseInner::Dynamic(ref platform) => Ok(&platform.url),
			RemoteReleaseInner::Static { ref platforms } => platforms
				.get(target)
				.map_or(Err(Error::TargetNotFound(target.to_string())), |p| Ok(&p.url))
		}
	}

	/// The signature for the release for the given target.
	pub fn signature(&self, target: &str) -> Result<&String> {
		match self.data {
			RemoteReleaseInner::Dynamic(ref platform) => Ok(&platform.signature),
			RemoteReleaseInner::Static { ref platforms } => platforms
				.get(target)
				.map_or(Err(Error::TargetNotFound(target.to_string())), |platform| Ok(&platform.signature))
		}
	}

	#[cfg(target_os = "windows")]
	/// Optional: Windows only try to use elevated task
	pub fn with_elevated_task(&self, target: &str) -> Result<bool> {
		match self.data {
			RemoteReleaseInner::Dynamic(ref platform) => Ok(platform.with_elevated_task),
			RemoteReleaseInner::Static { ref platforms } => platforms
				.get(target)
				.map_or(Err(Error::TargetNotFound(target.to_string())), |platform| Ok(platform.with_elevated_task))
		}
	}
}

pub struct UpdateBuilder<R: Runtime> {
	/// Application handle.
	pub app: AppHandle<R>,
	/// Current version we are running to compare with announced version
	pub current_version: Version,
	/// The URLs to checks updates. We suggest at least one fallback on a
	/// different domain.
	pub urls: Vec<String>,
	/// The platform the updater will check and install the update. Default is
	/// from `get_updater_target`
	pub target: Option<String>,
	/// The current executable path. Default is automatically extracted.
	pub executable_path: Option<PathBuf>,
	should_install: Option<Box<ShouldInstall>>,
	timeout: Option<Duration>,
	headers: HeaderMap
}

impl<R: Runtime> fmt::Debug for UpdateBuilder<R> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("UpdateBuilder")
			.field("app", &self.app)
			.field("current_version", &self.current_version)
			.field("urls", &self.urls)
			.field("target", &self.target)
			.field("executable_path", &self.executable_path)
			.field("timeout", &self.timeout)
			.field("headers", &self.headers)
			.finish()
	}
}

// Create new updater instance and return an Update
impl<R: Runtime> UpdateBuilder<R> {
	pub fn new(app: AppHandle<R>) -> Self {
		UpdateBuilder {
			app,
			urls: Vec::new(),
			target: None,
			executable_path: None,
			current_version: env!("CARGO_PKG_VERSION").parse().unwrap(),
			should_install: None,
			timeout: None,
			headers: Default::default()
		}
	}

	#[allow(dead_code)]
	pub fn url(mut self, url: String) -> Self {
		self.urls
			.push(percent_encoding::percent_decode(url.as_bytes()).decode_utf8_lossy().to_string());
		self
	}

	/// Add multiple URLS at once inside a Vec for future reference
	pub fn urls(mut self, urls: &[String]) -> Self {
		let mut formatted_vec: Vec<String> = Vec::new();
		for url in urls {
			formatted_vec.push(percent_encoding::percent_decode(url.as_bytes()).decode_utf8_lossy().to_string());
		}
		self.urls = formatted_vec;
		self
	}

	/// Set the current app version, used to compare against the latest
	/// available version. The `cargo_crate_version!` macro can be used to pull
	/// the version from your `Cargo.toml`
	pub fn current_version(mut self, ver: Version) -> Self {
		self.current_version = ver;
		self
	}

	/// Set the target name. Represents the string that is looked up on the updater API or response JSON.
	pub fn target(mut self, target: impl Into<String>) -> Self {
		self.target.replace(target.into());
		self
	}

	/// Set the executable path
	#[allow(dead_code)]
	pub fn executable_path<A: AsRef<Path>>(mut self, executable_path: A) -> Self {
		self.executable_path = Some(PathBuf::from(executable_path.as_ref()));
		self
	}

	pub fn should_install<F: FnOnce(&Version, &RemoteRelease) -> bool + Send + 'static>(mut self, f: F) -> Self {
		self.should_install.replace(Box::new(f));
		self
	}

	pub fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout.replace(timeout);
		self
	}

	/// Add a `Header` to the request.
	pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
	where
		HeaderName: TryFrom<K>,
		<HeaderName as TryFrom<K>>::Error: Into<http::Error>,
		HeaderValue: TryFrom<V>,
		<HeaderValue as TryFrom<V>>::Error: Into<http::Error>
	{
		let key: std::result::Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
		let value: std::result::Result<HeaderValue, http::Error> = value.try_into().map_err(Into::into);
		self.headers.insert(key?, value?);
		Ok(self)
	}

	pub async fn build(mut self) -> Result<Update<R>> {
		let mut remote_release: Option<RemoteRelease> = None;

		// make sure we have at least one url
		if self.urls.is_empty() {
			return Err(Error::Builder("Unable to check update, `url` is required.".into()));
		};

		// If no executable path provided, we use current_exe from millennium_utils
		let executable_path = self.executable_path.unwrap_or(current_exe()?);

		let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
		// `target` is the `{{target}}` variable we replace in the endpoint.
		// `json_target` is the value we search if the update server returns a JSON document with the `platforms` object.
		let (target, json_target) = if let Some(target) = self.target {
			(target.clone(), target)
		} else {
			let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
			(target.to_string(), format!("{}-{}", target, arch))
		};

		// Get the extract_path from the provided executable_path
		let extract_path = extract_path_from_executable(&self.app.state::<Env>(), &executable_path);

		// Set SSL certs for linux if they aren't available.
		// We do not require to recheck in the download_and_install as we use
		// ENV variables, we can expect them to be set for the second call.
		#[cfg(target_os = "linux")]
		{
			if env::var_os("SSL_CERT_FILE").is_none() {
				env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
			}
			if env::var_os("SSL_CERT_DIR").is_none() {
				env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
			}
		}

		let mut headers = self.headers;
		headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

		// Allow fallback if more than 1 urls is provided
		let mut last_error: Option<Error> = None;
		for url in &self.urls {
			// replace {{current_version}}, {{target}}, and {{arch}} in the provided URL.
			// for example:
			// https://releases.myapp.com/update/{{target}}/{{arch}}/{{current_version}}
			// will be translated into ->
			// https://releases.myapp.com/update/darwin/aarch64/1.0.0
			// The main objective is if the update URL is defined via the Cargo.toml
			// the URL will be generated dynamicly
			let fixed_link = url
				.replace("{{current_version}}", &self.current_version.to_string())
				.replace("{{target}}", &target)
				.replace("{{arch}}", arch);

			let mut request = HttpRequestBuilder::new("GET", &fixed_link)?.headers(headers.clone());
			if let Some(timeout) = self.timeout {
				request = request.timeout(timeout);
			}
			let resp = ClientBuilder::new().build()?.send(request).await;

			// If we got a success, we stop the loop
			// and we set our remote_release variable
			if let Ok(res) = resp {
				let res = res.read().await?;
				// got status code 2XX
				if StatusCode::from_u16(res.status).map_err(|e| Error::Builder(e.to_string()))?.is_success() {
					// if we got 204
					if StatusCode::NO_CONTENT.as_u16() == res.status {
						// return with `UpToDate` error
						// we should catch on the client
						return Err(Error::UpToDate);
					};
					// Convert the remote result to our local struct
					let built_release = serde_json::from_value(res.data).map_err(Into::into);
					// make sure all went well and the remote data is compatible
					// with what we need locally
					match built_release {
						Ok(release) => {
							last_error = None;
							remote_release = Some(release);
							break;
						}
						Err(err) => last_error = Some(err)
					}
				} // if status code is not 2XX we keep loopin' our urls
			}
		}

		// Last error is cleaned on success -- shouldn't be triggered if
		// we have a successful call
		if let Some(error) = last_error {
			return Err(error);
		}

		// Extracted remote metadata
		let final_release = remote_release.ok_or(Error::ReleaseNotFound)?;

		// did the announced version is greated than our current one?
		let should_update = if let Some(comparator) = self.should_install.take() {
			comparator(&self.current_version, &final_release)
		} else {
			final_release.version() > &self.current_version
		};

		headers.remove("Accept");

		// create our new updater
		Ok(Update {
			app: self.app,
			target,
			extract_path,
			should_update,
			version: final_release.version().to_string(),
			date: final_release.pub_date().cloned(),
			current_version: self.current_version,
			download_url: final_release.download_url(&json_target)?.to_owned(),
			body: final_release.notes().cloned(),
			signature: final_release.signature(&json_target)?.to_owned(),
			#[cfg(target_os = "windows")]
			with_elevated_task: final_release.with_elevated_task(&json_target)?,
			timeout: self.timeout,
			headers
		})
	}
}

pub fn builder<R: Runtime>(app: AppHandle<R>) -> UpdateBuilder<R> {
	UpdateBuilder::new(app)
}

#[derive(Debug)]
pub struct Update<R: Runtime> {
	/// Application handle.
	pub app: AppHandle<R>,
	/// Update description
	pub body: Option<String>,
	/// Should we update or not
	pub should_update: bool,
	/// Version announced
	pub version: String,
	/// Running version
	pub current_version: Version,
	/// Update publish date
	pub date: Option<OffsetDateTime>,
	/// Target
	#[allow(dead_code)]
	target: String,
	/// Extract path
	extract_path: PathBuf,
	/// Download URL announced
	download_url: Url,
	/// Signature announced
	signature: String,
	#[cfg(target_os = "windows")]
	/// Optional: Windows only try to use elevated task
	/// Default to false
	with_elevated_task: bool,
	/// Request timeout
	timeout: Option<Duration>,
	/// Request headers
	headers: HeaderMap
}

impl<R: Runtime> Clone for Update<R> {
	fn clone(&self) -> Self {
		Update {
			app: self.app.clone(),
			body: self.body.clone(),
			should_update: self.should_update,
			version: self.version.clone(),
			current_version: self.current_version.clone(),
			date: self.date,
			target: self.target.clone(),
			extract_path: self.extract_path.clone(),
			download_url: self.download_url.clone(),
			signature: self.signature.clone(),
			#[cfg(target_os = "windows")]
			with_elevated_task: self.with_elevated_task,
			timeout: self.timeout,
			headers: self.headers.clone()
		}
	}
}

impl<R: Runtime> Update<R> {
	// Download and install our update
	// @todo(lemarier): Split into download and install (two step) but need to be
	// thread safe
	pub async fn download_and_install<C: Fn(usize, Option<u64>), D: FnOnce()>(&self, pub_key: String, on_chunk: C, on_download_finish: D) -> Result {
		// make sure we can install the update on linux
		// We fail here because later we can add more linux support
		// actually if we use APPIMAGE, our extract path should already
		// be set with our APPIMAGE env variable, we don't need to do
		// anythin with it yet
		#[cfg(target_os = "linux")]
		if self.app.state::<Env>().appimage.is_none() {
			return Err(Error::UnsupportedLinuxPackage);
		}

		// set our headers
		let mut headers = self.headers.clone();
		headers.insert("Accept", HeaderValue::from_str("application/octet-stream").unwrap());
		headers.insert("User-Agent", HeaderValue::from_str("millennium/updater").unwrap());

		let client = ClientBuilder::new().build()?;
		// Create our request
		let mut req = HttpRequestBuilder::new("GET", self.download_url.as_str())?.headers(headers);
		if let Some(timeout) = self.timeout {
			req = req.timeout(timeout);
		}

		let response = client.send(req).await?;

		// make sure it's success
		if !response.status().is_success() {
			return Err(Error::Network(format!("Download request failed with status: {}", response.status())));
		}

		let content_length: Option<u64> = response
			.headers()
			.get("Content-Length")
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.parse().ok());

		let mut buffer = Vec::new();
		let mut stream = response.bytes_stream();
		while let Some(chunk) = stream.next().await {
			let chunk = chunk?;
			let bytes = chunk.as_ref().to_vec();
			on_chunk(bytes.len(), content_length);
			buffer.extend(bytes);
		}

		on_download_finish();

		// create memory buffer from our archive (Seek + Read)
		let mut archive_buffer = Cursor::new(buffer);

		// we make sure the archive is valid and signed with the private key linked with
		// the publickey
		verify_signature(&mut archive_buffer, &self.signature, &pub_key)?;

		#[cfg(feature = "updater")]
		{
			// we copy the files depending of the operating system
			// we run the setup, appimage re-install or overwrite the
			// macos .app
			#[cfg(target_os = "windows")]
			copy_files_and_run(
				archive_buffer,
				&self.extract_path,
				self.with_elevated_task,
				self.app.config().millennium.updater.windows.install_mode.clone().msiexec_args()
			)?;
			#[cfg(not(target_os = "windows"))]
			copy_files_and_run(archive_buffer, &self.extract_path)?;
		}
		// We are done!
		Ok(())
	}
}

// Linux (AppImage)

// ### Expected structure:
// ├── [AppName]_[version]_amd64.AppImage.tar.gz
// │   └──[AppName]_[version]_amd64.AppImage
// └── ...

// We should have an AppImage already installed to be able to copy and install
// the extract_path is the current AppImage path
// tmp_dir is where our new AppImage is found
#[cfg(feature = "updater")]
#[cfg(target_os = "linux")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
	use std::os::unix::fs::PermissionsExt;
	let tmp_dir = tempfile::Builder::new().prefix("millennium_current_app").tempdir()?;
	let mut perms = std::fs::metadata(tmp_dir.path())?.permissions();
	perms.set_mode(0o700);
	std::fs::set_permissions(tmp_dir.path(), perms)?;

	let tmp_app_image = &tmp_dir.path().join("current_app.AppImage");

	// create a backup of our current app image
	Move::from_source(extract_path).to_dest(tmp_app_image)?;

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));
	extractor.with_files(|entry| {
		let path = entry.path()?;
		if path.extension() == Some(OsStr::new("AppImage")) {
			// if something went wrong during the extraction, we should restore previous app
			if let Err(err) = entry.extract(extract_path) {
				Move::from_source(tmp_app_image).to_dest(extract_path)?;
				return Err(crate::api::Error::Extract(err.to_string()));
			}
			// early finish we have everything we need here
			return Ok(true);
		}
		Ok(false)
	})?;

	Ok(())
}

// Windows

// ### Expected structure:
// ├── [AppName]_[version]_x64.msi.zip
// │   └──[AppName]_[version]_x64.msi
// └── ...

// ## MSI
// Update server can provide a MSI for Windows. (Generated with
// millennium-bundler from *Wix*) To replace current version of the application.
// In later version we'll offer incremental update to push specific binaries.

// ## EXE
// Update server can provide a custom EXE (installer) who can run any task.
#[cfg(feature = "updater")]
#[cfg(target_os = "windows")]
#[allow(clippy::unnecessary_wraps)]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, _extract_path: &Path, with_elevated_task: bool, msiexec_args: &[&str]) -> Result {
	// FIXME: We need to create a memory buffer with the MSI and then run it.
	//        (instead of extracting the MSI to a temp path)
	//
	// The tricky part is the MSI need to be exposed and spawned so the memory
	// allocation shouldn't drop but we should be able to pass the reference so we
	// can drop it once the installation is done, otherwise we have a huge memory
	// leak.

	let tmp_dir = tempfile::Builder::new().tempdir()?.into_path();

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Zip);

	// extract the msi
	extractor.extract_into(&tmp_dir)?;

	let paths = read_dir(&tmp_dir)?;
	// This consumes the TempDir without deleting directory on the filesystem,
	// meaning that the directory will no longer be automatically deleted.

	for path in paths {
		let found_path = path?.path();
		// we support 2 type of files exe & msi for now
		// If it's an `exe` we expect an installer not a runtime.
		if found_path.extension() == Some(OsStr::new("exe")) {
			// Run the EXE
			Command::new(found_path).spawn().expect("installer failed to start");

			exit(0);
		} else if found_path.extension() == Some(OsStr::new("msi")) {
			if with_elevated_task {
				if let Some(bin_name) = current_exe()
					.ok()
					.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
					.and_then(|s| s.into_string().ok())
				{
					let product_name = bin_name.replace(".exe", "");

					// Check if there is a task that enables the updater to skip the UAC prompt
					let update_task_name = format!("Update {} - Skip UAC", product_name);
					if let Ok(output) = Command::new("schtasks").arg("/QUERY").arg("/TN").arg(update_task_name.clone()).output() {
						if output.status.success() {
							// Rename the MSI to the match file name the Skip UAC task is expecting it to be
							let temp_msi = tmp_dir.with_file_name(bin_name).with_extension("msi");
							Move::from_source(&found_path).to_dest(&temp_msi).expect("Unable to move update MSI");
							let exit_status = Command::new("schtasks")
								.arg("/RUN")
								.arg("/TN")
								.arg(update_task_name)
								.status()
								.expect("failed to start updater task");

							if exit_status.success() {
								// Successfully launched task that skips the UAC prompt
								exit(0);
							}
						}
						// Failed to run update task. Following UAC Path
					}
				}
			}

			// restart should be handled by WIX as we exit the process
			Command::new("msiexec.exe")
				.arg("/i")
				.arg(found_path)
				.args(msiexec_args)
				.arg("/promptrestart")
				.spawn()
				.expect("installer failed to start");

			exit(0);
		}
	}

	Ok(())
}

// MacOS
// ### Expected structure:
// ├── [AppName]_[version]_x64.app.tar.gz
// │   └──[AppName].app
// │      └── Contents
// │          └── ...
// └── ...
#[cfg(feature = "updater")]
#[cfg(target_os = "macos")]
fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
	let mut extracted_files: Vec<PathBuf> = Vec::new();

	// extract the buffer to the tmp_dir
	// we extract our signed archive into our final directory without any temp file
	let mut extractor = Extract::from_cursor(archive_buffer, ArchiveFormat::Tar(Some(Compression::Gz)));
	// the first file in the tar.gz will always be
	// <app_name>/Contents
	let tmp_dir = tempfile::Builder::new().prefix("millennium_current_app").tempdir()?;

	// create backup of our current app
	Move::from_source(extract_path).to_dest(tmp_dir.path())?;

	// extract all the files
	extractor.with_files(|entry| {
		let path = entry.path()?;
		// skip the first folder (should be the app name)
		let collected_path: PathBuf = path.iter().skip(1).collect();
		let extraction_path = extract_path.join(collected_path);

		// if something went wrong during the extraction, we should restore previous app
		if let Err(err) = entry.extract(&extraction_path) {
			for file in &extracted_files {
				// delete all the files we extracted
				if file.is_dir() {
					std::fs::remove_dir(file)?;
				} else {
					std::fs::remove_file(file)?;
				}
			}
			Move::from_source(tmp_dir.path()).to_dest(extract_path)?;
			return Err(crate::api::Error::Extract(err.to_string()));
		}

		extracted_files.push(extraction_path);

		Ok(false)
	})?;

	Ok(())
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
	if cfg!(target_os = "linux") {
		Some("linux")
	} else if cfg!(target_os = "macos") {
		Some("darwin")
	} else if cfg!(target_os = "windows") {
		Some("windows")
	} else {
		None
	}
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
	if cfg!(target_arch = "x86_64") {
		Some("x64")
	} else if cfg!(target_arch = "x86") {
		Some("x86")
	} else if cfg!(target_arch = "arm") {
		Some("armv7")
	} else if cfg!(target_arch = "aarch64") {
		Some("aarch64")
	} else {
		None
	}
}

/// Get the extract_path from the provided executable_path
#[allow(unused_variables)]
pub fn extract_path_from_executable(env: &Env, executable_path: &Path) -> PathBuf {
	// Return the path of the current executable by default
	// Example C:\Program Files\My App\
	let extract_path = executable_path.parent().map(PathBuf::from).expect("Can't determine extract path");

	// MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
	// We need to get /Applications/<app>.app
	// todo(lemarier): Need a better way here
	// Maybe we could search for <*.app> to get the right path
	#[cfg(target_os = "macos")]
	if extract_path.display().to_string().contains("Contents/MacOS") {
		return extract_path
			.parent()
			.map(PathBuf::from)
			.expect("Unable to find the extract path")
			.parent()
			.map(PathBuf::from)
			.expect("Unable to find the extract path");
	}

	// We should use APPIMAGE exposed env variable
	// This is where our APPIMAGE should sit and should be replaced
	#[cfg(target_os = "linux")]
	if let Some(app_image_path) = &env.appimage {
		return PathBuf::from(app_image_path);
	}

	extract_path
}

// Convert base64 to string and prevent failing
fn base64_to_string(base64_string: &str) -> Result<String> {
	let decoded_string = &decode(base64_string)?;
	let result = from_utf8(decoded_string)
		.map_err(|_| Error::SignatureUtf8(base64_string.into()))?
		.to_string();
	Ok(result)
}

// Validate signature
// need to be public because its been used
// by our tests in the bundler
//
// NOTE: The buffer position is not reset.
pub fn verify_signature<R>(archive_reader: &mut R, release_signature: &str, pub_key: &str) -> Result<bool>
where
	R: Read
{
	// we need to convert the pub key
	let pub_key_decoded = base64_to_string(pub_key)?;
	let public_key = PublicKey::decode(&pub_key_decoded)?;
	let signature_base64_decoded = base64_to_string(release_signature)?;
	let signature = Signature::decode(&signature_base64_decoded)?;

	// read all bytes until EOF in the buffer
	let mut data = Vec::new();
	archive_reader.read_to_end(&mut data)?;

	// Validate signature or bail out
	public_key.verify(&data, &signature, true)?;
	Ok(true)
}
