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

//! Types and functions related to file operations.

#[cfg(feature = "fs-extract-api")]
mod extract;
mod file_move;

use std::{
	fs,
	path::{Display, Path}
};

#[cfg(feature = "fs-extract-api")]
pub use extract::*;
pub use file_move::*;
use serde::{de::Error as DeError, Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub(crate) struct SafePathBuf(std::path::PathBuf);

impl SafePathBuf {
	pub fn new(path: std::path::PathBuf) -> Result<Self, &'static str> {
		if path.components().any(|x| matches!(x, std::path::Component::ParentDir)) {
			Err("cannot traverse directory, rewrite the path without the use of `../`")
		} else {
			Ok(Self(path))
		}
	}

	#[allow(dead_code)]
	pub unsafe fn new_unchecked(path: std::path::PathBuf) -> Self {
		Self(path)
	}

	#[allow(dead_code)]
	pub fn display(&self) -> Display<'_> {
		self.0.display()
	}
}

impl AsRef<Path> for SafePathBuf {
	fn as_ref(&self) -> &Path {
		self.0.as_ref()
	}
}

impl<'de> Deserialize<'de> for SafePathBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>
	{
		let path = std::path::PathBuf::deserialize(deserializer)?;
		SafePathBuf::new(path).map_err(DeError::custom)
	}
}

/// Reads the entire contents of a file into a string.
pub fn read_string<P: AsRef<Path>>(file: P) -> crate::api::Result<String> {
	fs::read_to_string(file).map_err(Into::into)
}

/// Reads the entire contents of a file into a bytes vector.
pub fn read_binary<P: AsRef<Path>>(file: P) -> crate::api::Result<Vec<u8>> {
	fs::read(file).map_err(Into::into)
}

#[cfg(test)]
mod test {
	use std::path::PathBuf;

	use quickcheck::{Arbitrary, Gen};

	use super::*;
	#[cfg(not(windows))]
	use crate::api::Error;

	impl Arbitrary for super::SafePathBuf {
		fn arbitrary(g: &mut Gen) -> Self {
			Self(PathBuf::arbitrary(g))
		}

		fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
			Box::new(self.0.shrink().map(SafePathBuf))
		}
	}

	#[test]
	fn check_read_string() {
		let file = String::from("test/api/test.txt");

		let res = read_string(file);

		assert!(res.is_ok());

		if let Ok(s) = res {
			assert_eq!(s, "This is a test doc!\n".to_string());
		}
	}

	#[test]
	#[allow(clippy::assertions_on_result_states)]
	fn check_read_string_fail() {
		let file = String::from("test/api/");

		let res = read_string(file);

		assert!(res.is_err());

		#[cfg(not(windows))]
		if let Error::Io(e) = res.unwrap_err() {
			#[cfg(not(windows))]
			assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
		}
	}

	#[test]
	fn check_read_binary() {
		let file = String::from("test/api/test_binary");

		let expected_vec = vec![
			71, 73, 70, 56, 57, 97, 1, 0, 1, 0, 128, 0, 0, 255, 255, 255, 0, 0, 0, 33, 249, 4, 1, 0, 0, 0, 0, 44, 0, 0, 0, 0, 1, 0, 1, 0, 0, 2, 2, 68, 1, 0, 59,
		];

		let res = read_binary(file);

		assert!(res.is_ok());

		if let Ok(vec) = res {
			assert_eq!(vec, expected_vec);
		}
	}

	#[test]
	#[allow(clippy::assertions_on_result_states)]
	fn check_read_binary_fail() {
		let file = String::from("test/api/");

		let res = read_binary(file);

		assert!(res.is_err());

		#[cfg(not(windows))]
		if let Error::Io(e) = res.unwrap_err() {
			#[cfg(not(windows))]
			assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
		}
	}
}
