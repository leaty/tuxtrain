use crate::error::MemError;
use crate::Pattern;
use nix::errno::Errno;
use nix::sys::uio::{process_vm_readv, process_vm_writev};
use nix::sys::uio::{IoVec, RemoteIoVec};
use nix::unistd::Pid;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, Write};

const CHUNK_SIZE: usize = 1024;

pub fn search(
	pid: &Pid,
	region: &(usize, usize),
	pattern: &Pattern,
) -> Result<(usize, Vec<u8>), MemError> {
	let end = region.1;
	let find = pattern.len();
	let mut chunk_size = CHUNK_SIZE;
	let mut pointer = region.0;
	let mut found = vec![];
	let mut at = 0;

	loop {
		// Avoid overreach
		if pointer + chunk_size > end {
			chunk_size = end - pointer;
		}

		// Read memory region one chunk at a time
		let chunk = read(pid, pointer, chunk_size)?;

		// Go through chunk per byte, forward-match with pattern each time
		for i in 0..chunk.len() {
			chunk
				.iter()
				.skip(i)
				.zip(pattern.iter().skip(found.len()))
				.all(|(mbyte, cbyte)| {
					match cbyte {
						// Store matching bytes
						// None "__" criteria always matches
						Some(b) if b == mbyte => found.push(*mbyte),
						None => found.push(*mbyte),

						// Doesn't match, reset
						_ => {
							found.clear();
							return false;
						}
					};

					// Set "at" on first discovery
					if found.len() == 1 {
						at = pointer + i;
					}

					return true;
				});

			// Found what there is to find
			if found.len() == find {
				return Ok((at, found));
			}
		}

		// Set next chunk
		pointer += chunk.len();

		// End of region, never found it sadly
		if pointer == end {
			break;
		}
	}

	Err(MemError::Read("Could not find pattern '{pattern}'".into()))?
}

pub fn replace(pid: &Pid, at: usize, this: &Vec<u8>, with: &Pattern) -> Result<Vec<u8>, MemError> {
	if this.len() != with.len() {
		Err(MemError::Write("Replacement differs in length.".into()))?
	}

	// Merge "this" "with"
	// None: Use left
	// Some: Use right
	let mut replace = vec![];
	for (i, b) in this.iter().enumerate() {
		replace.push(with[i].unwrap_or_else(|| *b));
	}

	write(pid, at, &replace)?;

	Ok(replace)
}

pub fn read(pid: &Pid, at: usize, len: usize) -> Result<Vec<u8>, MemError> {
	let mut data = vec![0; len];
	let local = IoVec::<&mut [u8]>::from_mut_slice(&mut data);
	let remote = RemoteIoVec { base: at, len };

	process_vm_readv(*pid, &[local], &[remote]).map_err(|e| MemError::Read(e.to_string()))?;
	Ok(data)
}

pub fn write(pid: &Pid, at: usize, data: &Vec<u8>) -> Result<usize, MemError> {
	let local = IoVec::<&[u8]>::from_slice(data);
	let remote = RemoteIoVec {
		base: at,
		len: data.len(),
	};

	// Try to write directly. On EFAULT, it's likely a
	// protected page so try ptrace instead as fallback
	match process_vm_writev(*pid, &[local], &[remote]) {
		Err(Errno::EFAULT) => {
			write_protected(pid, at as u64, data).map_err(|e| MemError::Write(e.to_string()))
		}
		r => r.map_err(|e| MemError::Write(e.to_string())),
	}
}

/// Write using ptrace and opening /proc/pid/mem directly.
/// Used for protected "read-only" pages.
fn write_protected(pid: &Pid, at: u64, data: &Vec<u8>) -> Result<usize, io::Error> {
	nix::sys::ptrace::attach(*pid)?;

	let mut f = File::options()
		.read(true)
		.write(true)
		.open(format!("/proc/{}/mem", pid.as_raw()))
		.unwrap();

	f.seek(SeekFrom::Start(at))?;
	let r = f.write(data);

	nix::sys::ptrace::detach(*pid, Some(nix::sys::signal::Signal::SIGCONT))?;
	r
}
