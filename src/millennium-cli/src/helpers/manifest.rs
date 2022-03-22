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
	collections::HashSet,
	fs::File,
	io::{Read, Write},
	iter::FromIterator,
	path::Path
};

use anyhow::Context;
use toml_edit::{Array, Document, InlineTable, Item, Table, Value};

use super::{
	app_paths::millennium_dir,
	config::{ConfigHandle, PatternKind}
};

#[derive(Default)]
pub struct Manifest {
	pub features: HashSet<String>
}

fn read_manifest(manifest_path: &Path) -> crate::Result<Document> {
	let mut manifest_str = String::new();

	let mut manifest_file = File::open(manifest_path).with_context(|| format!("failed to open `{:?}` file", manifest_path))?;
	manifest_file.read_to_string(&mut manifest_str)?;

	let manifest: Document = manifest_str.parse::<Document>().with_context(|| "failed to parse Cargo.toml")?;

	Ok(manifest)
}

fn toml_array(features: &HashSet<String>) -> Array {
	let mut f = Array::default();
	let mut features: Vec<String> = features.iter().map(|f| f.to_string()).collect();
	features.sort();
	for feature in features {
		f.push(feature.as_str());
	}
	f
}

fn write_features(dependencies: &mut Table, dependency_name: &str, all_features: Vec<&str>, features: &mut HashSet<String>) -> crate::Result<bool> {
	let item = dependencies.entry(dependency_name).or_insert(Item::None);

	if let Some(dep) = item.as_table_mut() {
		let manifest_features = dep.entry("features").or_insert(Item::None);
		if let Item::Value(Value::Array(f)) = &manifest_features {
			for feat in f.iter() {
				if let Value::String(feature) = feat {
					if !all_features.contains(&feature.value().as_str()) {
						features.insert(feature.value().to_string());
					}
				}
			}
		}
		*manifest_features = Item::Value(Value::Array(toml_array(features)));
		Ok(true)
	} else if let Some(dep) = item.as_value_mut() {
		match dep {
			Value::InlineTable(table) => {
				let manifest_features = table.get_or_insert("features", Value::Array(Default::default()));
				if let Value::Array(f) = &manifest_features {
					for feat in f.iter() {
						if let Value::String(feature) = feat {
							if !all_features.contains(&feature.value().as_str()) {
								features.insert(feature.value().to_string());
							}
						}
					}
				}
				*manifest_features = Value::Array(toml_array(features));
			}
			Value::String(version) => {
				let mut def = InlineTable::default();
				def.get_or_insert("version", version.to_string().replace('\"', "").replace(' ', ""));
				def.get_or_insert("features", Value::Array(toml_array(features)));
				*dep = Value::InlineTable(def);
			}
			_ => return Err(anyhow::anyhow!("Unsupported {} dependency format on Cargo.toml", dependency_name))
		}
		Ok(true)
	} else {
		Ok(false)
	}
}

pub fn rewrite_manifest(config: ConfigHandle) -> crate::Result<Manifest> {
	let manifest_path = millennium_dir().join("Cargo.toml");
	let mut manifest = read_manifest(&manifest_path)?;

	let config_guard = config.lock().unwrap();
	let config = config_guard.as_ref().unwrap();

	let mut millennium_build_features = HashSet::new();
	if let PatternKind::Isolation { .. } = config.millennium.pattern {
		millennium_build_features.insert("isolation".to_string());
	}
	let resp = write_features(
		manifest
			.as_table_mut()
			.entry("build-dependencies")
			.or_insert(Item::Table(Table::new()))
			.as_table_mut()
			.expect("manifest build-dependencies isn't a table"),
		"millennium-build",
		vec!["isolation"],
		&mut millennium_build_features
	)?;

	let mut millennium_features = HashSet::from_iter(config.millennium.features().into_iter().map(|f| f.to_string()));
	let cli_managed_millennium_features = super::config::MillenniumConfig::all_features();
	let res = match write_features(
		manifest
			.as_table_mut()
			.entry("dependencies")
			.or_insert(Item::Table(Table::new()))
			.as_table_mut()
			.expect("manifest dependencies isn't a table"),
		"millennium",
		cli_managed_millennium_features,
		&mut millennium_features
	) {
		Err(e) => Err(e),
		Ok(t) if !resp => Ok(t),
		_ => Ok(true)
	};

	match res {
		Ok(true) => {
			let mut manifest_file = File::create(&manifest_path).with_context(|| "failed to open Cargo.toml for rewrite")?;
			manifest_file.write_all(
				manifest
          .to_string()
          // apply some formatting fixes
          .replace(r#"" ,features =["#, r#"", features = ["#)
          .replace(r#"" , features"#, r#"", features"#)
          .replace("]}", "] }")
          .replace("={", "= {")
          .replace("=[", "= [")
          .as_bytes()
			)?;
			manifest_file.flush()?;
			Ok(Manifest { features: millennium_features })
		}
		Ok(false) => Ok(Manifest { features: millennium_features }),
		Err(e) => Err(e)
	}
}

pub fn get_workspace_members() -> crate::Result<Vec<String>> {
	let mut manifest = read_manifest(&millennium_dir().join("Cargo.toml"))?;
	let workspace = manifest.as_table_mut().entry("workspace").or_insert(Item::None).as_table_mut();

	match workspace {
		Some(workspace) => {
			let members = workspace
				.entry("members")
				.or_insert(Item::None)
				.as_array()
				.expect("workspace members aren't an array");
			Ok(members.iter().map(|v| v.as_str().unwrap().to_string()).collect())
		}
		None => Ok(vec![])
	}
}
