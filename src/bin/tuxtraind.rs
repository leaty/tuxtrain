//! TODO Remove WETness, since this is mostly a modified copy from tuxtrain bin.
use clap::Parser;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tuxtrain::{trainer, Trainer};

const TRAINER_DIR: &str = "/etc/tuxtrain";

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	#[clap(help = r#"Trainer to run, based on /etc/tuxtrain/<TRAINER>.toml
Omit this field to run all trainers"#)]
	trainer: Option<String>,
	#[clap(short, long, help = "Load trainer directly from file")]
	trainer_file: Option<PathBuf>,
	#[clap(short, long, help = "Don't print messages")]
	silent: bool,
	#[clap(short, long, help = "Full memory scan, ignoring region presets")]
	full: bool,
	#[clap(
		short,
		long,
		help = "Rate (ms) to probe processes",
		default_value_t = 1000
	)]
	rate: u64,
}

fn main() -> Result<(), Box<dyn Error>> {
	let dir = Path::new(TRAINER_DIR);
	let args = Args::parse();
	let rate = Duration::from_millis(args.rate);
	let trainers = trainer::from_args(dir, args.trainer, args.trainer_file)?;
	let mut status = HashSet::new();

	loop {
		for (idx, trainer) in trainers.iter().enumerate() {
			if trainer.enable {
				// When process is found and it hasn't
				// already been run for this instance
				if trainer.probe().is_ok() {
					if !status.contains(&idx) {
						delay(trainer);
						trainer.run(args.full, !args.silent);
						status.insert(idx);
					}
				// Process disappeared
				} else {
					status.remove(&idx);
				}
			}
		}

		thread::sleep(rate);
	}
}

/// Waits according to trainer.daemon.delay if any
fn delay(trainer: &Trainer) {
	if let Some(daemon) = &trainer.daemon {
		if let Some(delay) = daemon.delay {
			thread::sleep(Duration::from_millis(delay));
		}
	}
}
