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