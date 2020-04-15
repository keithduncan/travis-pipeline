use core::str::FromStr;

use std::{
	collections::HashMap as Map,
};

use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Travis {
    pub language: String,
	pub rust: Vec<String>,
	pub env: Vec<String>,
	pub script: Vec<String>,
	pub matrix: Option<Map<String, Vec<Map<String, String>>>>,
}

pub enum Rust {
    Stable,
    Beta,
    Nightly,
    // SemVer release number
    Release(String),
}

impl Rust {
    // Get a docker image for this travis rust tag
    pub fn image(&self) -> String {
    	match self {
    		&Rust::Stable => "rust:latest".to_string(),
    		&Rust::Beta => unimplemented!(),
    		&Rust::Nightly => "rustlang/rust:nightly".to_string(),
    		&Rust::Release(ref ver) => format!("rust:{}", ver),
    	}
    }
}

impl FromStr for Rust {
	type Err = ();

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(match s {
			"stable" => Rust::Stable,
			"beta" => Rust::Beta,
			"nighyly" => Rust::Nightly,
			v => Rust::Release(v.to_string()),
		})
	}
}
