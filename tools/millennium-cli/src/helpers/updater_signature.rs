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
	env::var_os,
	fs::{self, File, OpenOptions},
	io::{BufReader, BufWriter, Write},
	path::{Path, PathBuf},
	str,
	time::{SystemTime, UNIX_EPOCH}
};

use anyhow::Context;
use base64::{decode, encode};
use minisign::{sign, KeyPair as KP, SecretKeyBox};

/// A key pair (`PublicKey` and `SecretKey`).
#[derive(Clone, Debug)]
pub struct KeyPair {
	pub pk: String,
	pub sk: String
}

fn create_file(path: &Path) -> crate::Result<BufWriter<File>> {
	if let Some(parent) = path.parent() {
		fs::create_dir_all(&parent)?;
	}
	let file = File::create(path)?;
	Ok(BufWriter::new(file))
}

/// Generate base64 encoded keypair
pub fn generate_key(password: Option<String>) -> crate::Result<KeyPair> {
	let KP { pk, sk } = KP::generate_encrypted_keypair(password).unwrap();

	let pk_box_str = pk.to_box().unwrap().to_string();
	let sk_box_str = sk.to_box(None).unwrap().to_string();

	let encoded_pk = encode(&pk_box_str);
	let encoded_sk = encode(&sk_box_str);

	Ok(KeyPair { pk: encoded_pk, sk: encoded_sk })
}

/// Transform a base64 String to readable string for the main signer
pub fn decode_key(base64_key: String) -> crate::Result<String> {
	let decoded_str = &decode(&base64_key)?[..];
	Ok(String::from(str::from_utf8(decoded_str)?))
}

/// Save KeyPair to disk
pub fn save_keypair<P>(force: bool, sk_path: P, key: &str, pubkey: &str) -> crate::Result<(PathBuf, PathBuf)>
where
	P: AsRef<Path>
{
	let sk_path = sk_path.as_ref();

	let pubkey_path = format!("{}.pub", sk_path.display());
	let pk_path = Path::new(&pubkey_path);

	if sk_path.exists() {
		if !force {
			return Err(anyhow::anyhow!(
				"Key generation aborted:\n{} already exists\nIf you really want to overwrite the existing key pair, add the --force switch to force this operation.",
				sk_path.display()
			));
		} else {
			std::fs::remove_file(&sk_path)?;
		}
	}

	if pk_path.exists() {
		std::fs::remove_file(&pk_path)?;
	}

	let mut sk_writer = create_file(sk_path)?;
	write!(sk_writer, "{:}", key)?;
	sk_writer.flush()?;

	let mut pk_writer = create_file(pk_path)?;
	write!(pk_writer, "{:}", pubkey)?;
	pk_writer.flush()?;

	Ok((fs::canonicalize(&sk_path)?, fs::canonicalize(&pk_path)?))
}

/// Read key from file
pub fn read_key_from_file<P>(sk_path: P) -> crate::Result<String>
where
	P: AsRef<Path>
{
	Ok(fs::read_to_string(sk_path)?)
}

/// Sign files
pub fn sign_file<P>(private_key: String, password: Option<String>, bin_path: P) -> crate::Result<(PathBuf, String)>
where
	P: AsRef<Path>
{
	let bin_path = bin_path.as_ref();
	let decoded_secret = decode_key(private_key)?;
	let sk_box = SecretKeyBox::from_string(&decoded_secret).with_context(|| "failed to load updater private key")?;
	let sk = sk_box
		.into_secret_key(password)
		.with_context(|| "incorrect updater private key password")?;

	// We need to append .sig at the end it's where the signature will be stored
	let mut extension = bin_path.extension().unwrap().to_os_string();
	extension.push(".sig");
	let signature_path = bin_path.with_extension(extension);

	let mut signature_box_writer = create_file(&signature_path)?;

	let trusted_comment = format!("timestamp:{}\tfile:{}", unix_timestamp(), bin_path.file_name().unwrap().to_string_lossy());

	let data_reader = open_data_file(bin_path)?;

	let signature_box = sign(None, &sk, data_reader, Some(trusted_comment.as_str()), Some("signature from Millennium secret key"))?;

	let encoded_signature = encode(&signature_box.to_string());
	signature_box_writer.write_all(encoded_signature.as_bytes())?;
	signature_box_writer.flush()?;
	Ok((fs::canonicalize(&signature_path)?, encoded_signature))
}

/// Sign files using the MILLENNIUM_KEY_PASSWORD and MILLENNIUM_PRIVATE_KEY environment variables
pub fn sign_file_from_env_variables<P>(path_to_sign: P) -> crate::Result<(PathBuf, String)>
where
	P: AsRef<Path>
{
	// if no password provided we set empty string
	let password_string = var_os("MILLENNIUM_KEY_PASSWORD").map(|value| value.to_str().unwrap().to_string());
	// get the private key
	if let Some(private_key) = var_os("MILLENNIUM_PRIVATE_KEY") {
		// check if this file exist..
		let mut private_key_string = String::from(private_key.to_str().unwrap());
		let pk_dir = Path::new(&private_key_string);
		// Check if user provided a path or a key
		// We validate if the path exist or no.
		if pk_dir.exists() {
			// read file content as use it as private key
			private_key_string = read_key_from_file(pk_dir)?;
		}
		// sign our file
		sign_file(private_key_string, password_string, path_to_sign)
	} else {
		// reject if we don't have the private key
		Err(anyhow::anyhow!("A public key has been found, but no private key. Make sure to set `MILLENNIUM_PRIVATE_KEY` environment variable."))
	}
}

fn unix_timestamp() -> u64 {
	let start = SystemTime::now();
	let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("system clock is incorrect");
	since_the_epoch.as_secs()
}

fn open_data_file<P>(data_path: P) -> crate::Result<BufReader<File>>
where
	P: AsRef<Path>
{
	let data_path = data_path.as_ref();
	let file = OpenOptions::new()
		.read(true)
		.open(data_path)
		.map_err(|e| minisign::PError::new(minisign::ErrorKind::Io, e))?;
	Ok(BufReader::new(file))
}
