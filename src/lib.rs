// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The library used by the backup-rat utility that contains
//! all of the actual backup code
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod config;
pub mod operation;
