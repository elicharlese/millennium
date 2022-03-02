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

use proc_macro2::Ident;
use syn::{Path, PathSegment};

pub use self::{handler::Handler, wrapper::wrapper};

mod handler;
mod wrapper;

/// The autogenerated wrapper ident.
fn format_command_wrapper(function: &Ident) -> Ident {
	quote::format_ident!("__cmd__{}", function)
}

/// This function will panic if the passed [`syn::Path`] does not have any
/// segments.
fn path_to_command(path: &mut Path) -> &mut PathSegment {
	path.segments.last_mut().expect("parsed syn::Path has no segment")
}
