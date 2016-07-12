// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::fs;
use std::path::{PathBuf, Path};
use util::{H64, H256};
use util::journaldb::Algorithm;

// this const is irrelevent cause we do have migrations now,
// but we still use it for backwards compatibility
const LEGACY_CLIENT_DB_VER_STR: &'static str = "5.3";

#[derive(Debug, PartialEq)]
pub struct Directories {
	pub db: String,
	pub keys: String,
	pub signer: String,
	pub dapps: String,
}

impl Directories {
	pub fn create_dirs(&self) -> Result<(), String> {
		try!(fs::create_dir_all(&self.db).map_err(|e| e.to_string()));
		try!(fs::create_dir_all(&self.keys).map_err(|e| e.to_string()));
		try!(fs::create_dir_all(&self.signer).map_err(|e| e.to_string()));
		try!(fs::create_dir_all(&self.dapps).map_err(|e| e.to_string()));
		Ok(())
	}

	/// Get the path for the databases given the root path and information on the databases.
	pub fn client_path(&self, genesis_hash: H256, pruning: Algorithm) -> PathBuf {
		let mut dir = Path::new(&self.db).to_path_buf();
		dir.push(H64::from(genesis_hash).hex());
		dir.push(format!("v{}-sec-{}", LEGACY_CLIENT_DB_VER_STR, pruning));
		dir
	}
}
