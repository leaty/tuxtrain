use crate::error::HackError;
use crate::mem;
use crate::{pattern, Pattern};
use nix::unistd::Pid;
use procfs::process::Process;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Hack {
	pub name: String,
	pub enable: bool,
	#[serde(deserialize_with = "pattern::deserialize")]
	pub pattern: Pattern,
	#[serde(deserialize_with = "pattern::deserialize")]
	pub replace: Pattern,
	pub region: Option<(usize, usize)>,
}

impl Hack {
	pub fn run(&self, proc: &Process) -> HackResult {
		let pid = Pid::from_raw(proc.pid);
		let regions = match self.region.clone() {
			Some(r) => vec![r],
			// No predefined regions, use all
			None => proc
				.maps()?
				.iter()
				.map(|m| (m.address.0 as usize, m.address.1 as usize))
				.collect(),
		};

		for region in regions {
			if let Ok((at, found)) = mem::search(&pid, &region, &self.pattern) {
				let wrote = mem::replace(&pid, at, &found, &self.replace)?;
				return Ok(HackInfo { at, found, wrote });
			}
		}

		Err(HackError::Read("Unable to find pattern.".into()))
	}
}

pub struct HackInfo {
	pub at: usize,
	pub found: Vec<u8>,
	pub wrote: Vec<u8>,
}

pub type HackResult = Result<HackInfo, HackError>;
