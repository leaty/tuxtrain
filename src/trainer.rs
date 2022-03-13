pub mod daemon;

use crate::display;
use crate::error::TrainerError;
use crate::Hack;
use daemon::Daemon;
use procfs::{process, process::Process};
use serde::Deserialize;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Trainer {
	pub name: String,
	pub version: String,
	pub process: String,
	#[serde(rename = "feature")]
	pub features: Vec<Hack>,
	pub daemon: Option<Daemon>,
	pub enable: bool,
}

impl Trainer {
	pub fn run(&self, display: bool) {
		display.then(|| display::trainer(&self));

		let proc = self.probe();
		if let Ok(proc) = proc {
			for feature in &self.features {
				display.then(|| display::feature(feature));
				let res = feature.run(&proc);
				display.then(|| display::feature_result(&feature, &res));
			}
		} else if let Err(e) = proc {
			display.then(|| display::trainer_err(&e));
		}
	}

	pub fn probe(&self) -> Result<Process, TrainerError> {
		let procs = process::all_processes()?;
		for proc in procs {
			if proc.stat.comm == self.process {
				return Ok(proc);
			}
		}

		Err(TrainerError::Process(format!(
			"Could not find process by '{}'.",
			self.process
		)))?
	}

	pub fn features(&self) -> &Vec<Hack> {
		return &self.features;
	}
}

impl TryFrom<PathBuf> for Trainer {
	type Error = Box<dyn Error>;

	fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
		let trainer_str = std::fs::read_to_string(path)?;
		let trainer: Trainer = toml::from_str(&trainer_str)?;
		Ok(trainer)
	}
}

pub fn from_dir(dir: &Path) -> Result<Vec<Trainer>, Box<dyn Error>> {
	let mut trainers = vec![];

	let files = dir
		.read_dir()?
		.filter_map(|r| r.ok())
		.map(|e| e.path())
		.filter(|p| p.is_file());

	for file in files {
		trainers.push(file.clone().try_into()?);
	}

	Ok(trainers)
}

pub fn from_args(
	dir: &Path,
	trainer: Option<String>,
	trainer_file: Option<PathBuf>,
) -> Result<Vec<Trainer>, Box<dyn Error>> {
	Ok(if let Some(trainer) = trainer {
		vec![dir.join(format!("{}.toml", trainer)).try_into()?]
	} else if let Some(file) = trainer_file {
		vec![file.try_into()?]
	} else {
		from_dir(dir)?
	})
}
