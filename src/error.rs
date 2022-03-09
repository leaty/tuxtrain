use procfs::ProcError;
use std::fmt;

pub enum TrainerError {
	Process(String),
}

pub enum HackError {
	Read(String),
	Write(String),
}

pub enum MemError {
	Read(String),
	Write(String),
}

impl fmt::Display for TrainerError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			TrainerError::Process(e) => write!(f, "{}", e),
		}
	}
}

impl fmt::Display for HackError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			HackError::Read(e) => write!(f, "{}", e),
			HackError::Write(e) => write!(f, "{}", e),
		}
	}
}

impl From<ProcError> for TrainerError {
	fn from(err: ProcError) -> Self {
		TrainerError::Process(err.to_string())
	}
}

impl From<ProcError> for HackError {
	fn from(err: ProcError) -> Self {
		HackError::Read(err.to_string())
	}
}

impl From<MemError> for HackError {
	fn from(err: MemError) -> Self {
		match err {
			MemError::Read(e) => HackError::Read(e),
			MemError::Write(e) => HackError::Write(e),
		}
	}
}
