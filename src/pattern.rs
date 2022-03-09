use serde::de::Error;
use serde::de::{Deserialize, Deserializer};
use std::fmt;
use std::num::ParseIntError;
use std::ops::Index;
use std::slice::Iter;

pub struct Pattern {
	string: String,
	bytes: Vec<Option<u8>>,
}

impl Pattern {
	pub fn new(pattern: String) -> Result<Self, ParseIntError> {
		Ok(Self {
			bytes: as_bytes(&pattern)?,
			string: pattern,
		})
	}

	pub fn len(&self) -> usize {
		self.bytes.len()
	}

	pub fn iter(&self) -> Iter<'_, Option<u8>> {
		self.bytes.iter()
	}
}

impl fmt::Display for Pattern {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}", self.string)
	}
}

impl Index<usize> for Pattern {
	type Output = Option<u8>;

	fn index(&self, idx: usize) -> &Self::Output {
		&self.bytes[idx]
	}
}

pub fn as_bytes(pattern: &str) -> Result<Vec<Option<u8>>, ParseIntError> {
	let mut bytes = vec![];
	for hex in pattern.split(' ') {
		bytes.push(match hex {
			// __ is considered ignored
			"__" => None,
			_ => Some(u8::from_str_radix(hex, 16)?),
		});
	}

	Ok(bytes)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Pattern, D::Error>
where
	D: Deserializer<'de>,
{
	let pattern = String::deserialize(deserializer)?;
	Pattern::new(pattern).map_err(Error::custom)
}
