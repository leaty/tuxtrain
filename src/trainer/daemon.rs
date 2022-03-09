use serde::Deserialize;

/// Daemon options for trainer.
#[derive(Deserialize)]
pub struct Daemon {
	pub delay: Option<u64>,
}
