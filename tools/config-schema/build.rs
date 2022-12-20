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
	error::Error,
	fs::File,
	io::{BufWriter, Write},
	path::PathBuf
};

pub fn main() -> Result<(), Box<dyn Error>> {
	let schema = schemars::schema_for!(millennium_utils::config::Config);
	let schema_str = serde_json::to_string_pretty(&schema).unwrap();
	let crate_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);

	for file in [crate_dir.join("schema.json"), crate_dir.join("../millennium-cli/schema.json")] {
		let mut schema_file = BufWriter::new(File::create(&file)?);
		write!(schema_file, "{schema_str}")?;
	}

	Ok(())
}
