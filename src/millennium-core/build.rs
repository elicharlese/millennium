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
	// link carbon hotkey on macOS
	{
		if std::env::var("CARGO_CFG_TARGET_OS").map_or(false, |os| os == "macos") {
			println!("cargo:rustc-link-lib=framework=Carbon");
			cc::Build::new()
				.file("src/platform_impl/macos/carbon_hotkey/carbon_hotkey_binding.c")
				.compile("carbon_hotkey_binding.a");
		}
	}
}
