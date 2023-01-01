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

//! The Assets module allows you to read files that have been bundled by
//! Millennium during both compile time and runtime.

use std::{
	borrow::Cow,
	path::{Component, Path}
};

#[doc(hidden)]
pub use phf;

/// Represent an asset file path in a normalized way.
///
/// The following rules are enforced and added if needed:
/// * Unix path component separators
/// * Has a root directory
/// * No trailing slash - directories are not included in assets
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetKey(String);

impl From<AssetKey> for String {
	fn from(key: AssetKey) -> Self {
		key.0
	}
}

impl AsRef<str> for AssetKey {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl<P: AsRef<Path>> From<P> for AssetKey {
	fn from(path: P) -> Self {
		// TODO: change this to utilize `Cow` to prevent allocating an intermediate
		// `PathBuf` when not necessary
		let path = path.as_ref().to_owned();

		// add in root to mimic how it is used from a server url
		let path = if path.has_root() { path } else { Path::new(&Component::RootDir).join(path) };

		let buf = if cfg!(windows) {
			let mut buf = String::new();
			for component in path.components() {
				match component {
					Component::RootDir => buf.push('/'),
					Component::CurDir => buf.push_str("./"),
					Component::ParentDir => buf.push_str("../"),
					Component::Prefix(prefix) => buf.push_str(&prefix.as_os_str().to_string_lossy()),
					Component::Normal(s) => {
						buf.push_str(&s.to_string_lossy());
						buf.push('/')
					}
				}
			}

			// remove the last slash
			if buf != "/" {
				buf.pop();
			}

			buf
		} else {
			path.to_string_lossy().to_string()
		};

		AssetKey(buf)
	}
}

/// A Content-Security-Policy hash value for a specific directive.
/// For more information see [the MDN page](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy#directives).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum CspHash<'a> {
	/// The `script-src` directive.
	Script(&'a str),

	/// The `style-src` directive.
	Style(&'a str)
}

impl CspHash<'_> {
	/// The Content-Security-Policy directive this hash applies to.
	pub fn directive(&self) -> &'static str {
		match self {
			Self::Script(_) => "script-src",
			Self::Style(_) => "style-src"
		}
	}

	/// The value of the Content-Security-Policy hash.
	pub fn hash(&self) -> &str {
		match self {
			Self::Script(hash) => hash,
			Self::Style(hash) => hash
		}
	}
}

/// Represents a container of file assets that are retrievable during runtime.
pub trait Assets: Send + Sync + 'static {
	/// Get the content of the passed [`AssetKey`].
	fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>>;

	/// Gets the hashes for the CSP tag of the HTML on the given path.
	fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_>;
}

/// [`Assets`] implementation that only contains compile-time compressed and
/// embedded assets.
#[derive(Debug)]
pub struct EmbeddedAssets {
	assets: phf::Map<&'static str, &'static [u8]>,
	// Hashes that must be injected to the CSP of every HTML file.
	global_hashes: &'static [CspHash<'static>],
	// Hashes that are associated to the CSP of the HTML file identified by the map key (the HTML asset key).
	html_hashes: phf::Map<&'static str, &'static [CspHash<'static>]>
}

impl EmbeddedAssets {
	/// Creates a new instance from the given asset map and script hash list.
	pub const fn new(
		map: phf::Map<&'static str, &'static [u8]>,
		global_hashes: &'static [CspHash<'static>],
		html_hashes: phf::Map<&'static str, &'static [CspHash<'static>]>
	) -> Self {
		Self {
			assets: map,
			global_hashes,
			html_hashes
		}
	}
}

impl Assets for EmbeddedAssets {
	#[cfg(feature = "compression")]
	fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
		self.assets
			.get(key.as_ref())
			.map(|&(mut input_buf)| {
				// with the exception of extremely small files, output should usually be at least as large as the compressed version.
				let mut buf = Vec::with_capacity(input_buf.len());
				brotli::BrotliDecompress(&mut input_buf, &mut buf).map(|()| buf)
			})
			.and_then(Result::ok)
			.map(Cow::Owned)
	}

	#[cfg(not(feature = "compression"))]
	fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
		self.assets.get(key.as_ref()).copied().map(|a| Cow::Owned(a.to_vec()))
	}

	fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
		Box::new(
			self.global_hashes
				.iter()
				.chain(self.html_hashes.get(html_path.as_ref()).copied().into_iter().flatten())
				.copied()
		)
	}
}
