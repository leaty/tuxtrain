//! Improve this code someday.
//! But honestly, printing is always ugly.

use crate::{error::TrainerError, Hack, HackInfo, HackResult, Trainer};
use colored::*;

pub fn trainer(trainer: &Trainer) {
	println!(
		"{} {} {}",
		"==>".magenta().bold(),
		trainer.name.magenta().bold(),
		trainer.version.blue().bold()
	);
}

pub fn trainer_err(err: &TrainerError) {
	println!("\t{}: {}", "Notice".red().bold(), err);
}

pub fn feature(hack: &Hack) {
	println!("\t{}", hack.name.green().bold());
}

pub fn feature_result(hack: &Hack, result: &HackResult) {
	match result {
		Ok(info) => {
			println!("\t  - {}: {}", "Location".cyan(), info.at);
			println!(
				"\t  - {}: {}",
				"Found".cyan(),
				hex::encode(&info.found).to_uppercase()
			);
			println!(
				"\t  - {}: {}",
				"Wrote".cyan(),
				hex::encode(&info.wrote).to_uppercase()
			);

			feature_consider(hack, info);
		}
		Err(e) => {
			println!("\t  - {}: {}", "Error".red().bold(), e)
		}
	}
}

pub fn feature_consider(hack: &Hack, info: &HackInfo) {
	let mut consider = true;
	if let Some(region) = hack.region {
		// Region is already as tight as it gets
		if info.at == region.0 && info.at + info.found.len() == region.1 {
			consider = false;
		}
	};

	if consider {
		println!(
			"\t  - {}: [{}, {}].",
			"Consider tightening region".cyan(),
			info.at,
			info.at + info.found.len(),
		);
	}
}
