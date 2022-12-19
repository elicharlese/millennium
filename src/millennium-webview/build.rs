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

fn main() {
	let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
	if target_os == "macos" || target_os == "ios" {
		println!("cargo:rustc-link-lib=framework=WebKit");
	}

	let is_android = std::env::var("CARGO_CFG_TARGET_OS").map(|t| t == "android").unwrap_or_default();
	if is_android {
		use std::{fs, path::PathBuf};

		fn env_var(var: &str) -> String {
			std::env::var(var).unwrap_or_else(|_| panic!("`{var}` is not set, which is required to generate Kotlin files for Android."))
		}

		println!("cargo:rerun-if-env-changed=MILLENNIUM_ANDROID_REVERSED_DOMAIN");
		println!("cargo:rerun-if-env-changed=MILLENNIUM_ANDROID_APP_NAME_SNAKE_CASE");
		println!("cargo:rerun-if-env-changed=MILLENNIUM_ANDROID_KOTLIN_FILES_OUT_DIR");

		if let Ok(kotlin_out_dir) = std::env::var("MILLENNIUM_ANDROID_KOTLIN_FILES_OUT_DIR") {
			let reversed_domain = env_var("MILLENNIUM_ANDROID_REVERSED_DOMAIN");
			let app_name_snake_case = env_var("MILLENNIUM_ANDROID_APP_NAME_SNAKE_CASE");

			let kotlin_out_dir = PathBuf::from(kotlin_out_dir).canonicalize().expect("Failed to canonicalize path");
			let kotlin_files_path = PathBuf::from(env_var("CARGO_MANIFEST_DIR")).join("src/webview/android/kotlin");
			println!("cargo:rerun-if-changed={}", kotlin_files_path.display());
			let kotlin_files = fs::read_dir(kotlin_files_path).expect("failed to read kotlin directory");

			for file in kotlin_files {
				let file = file.unwrap();

				let class_extension_env = format!("MILLENNIUM_{}_CLASS_EXTENSION", file.path().file_stem().unwrap().to_string_lossy().to_uppercase());
				let class_init_env = format!("MILLENNIUM_{}_CLASS_INIT", file.path().file_stem().unwrap().to_string_lossy().to_uppercase());

				println!("cargo:rerun-if-env-changed={class_extension_env}");
				println!("cargo:rerun-if-env-changed={class_init_env}");

				let content = fs::read_to_string(file.path())
					.expect("failed to read kotlin file as string")
					.replace("{{app-domain-reversed}}", &reversed_domain)
					.replace("{{app-name-snake-case}}", &app_name_snake_case)
					.replace("{{class-extension}}", &std::env::var(&class_extension_env).unwrap_or_default())
					.replace("{{class-init}}", &std::env::var(&class_init_env).unwrap_or_default());

				fs::write(kotlin_out_dir.join(file.file_name()), content).expect("Failed to write kotlin file");
			}
		}
	}
}
