use clap::Parser;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use tuxtrain::trainer;

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
}

fn main() -> Result<(), Box<dyn Error>> {
	let dir = Path::new(TRAINER_DIR);
	let args = Args::parse();
	let trainers = trainer::from_args(dir, args.trainer, args.trainer_file)?;

	// Run trainers
	for trainer in trainers {
		if trainer.enable {
			trainer.run(args.full, !args.silent);
		}
	}

	Ok(())
}
