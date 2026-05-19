use std::{io, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
	#[error("I/O error: {0}")]
	Io(#[from] io::Error),

	#[error("JSON error: {0}")]
	Json(#[from] serde_json::Error),

	#[error("asar archive is truncated")]
	Truncated,

	#[error("file {0} already exists in archive")]
	FileAlreadyWritten(PathBuf),

	#[error("hash mismatch in file {file}")]
	HashMismatch {
		file: PathBuf,
		block: Option<usize>,
		expected: Vec<u8>,
		actual: Vec<u8>,
	},

	#[error("failed to read unpacked file {path}: {err}")]
	UnpackedIo { path: PathBuf, err: io::Error },

	#[error("unpacked file not found: {0}")]
	UnpackedFileNotFound(PathBuf),

	#[error("invalid hash algorithm: {0}")]
	InvalidHashAlgorithm(String),

	#[error("invalid path: {0}")]
	InvalidPath(PathBuf),

	#[error("header size {size} exceeds maximum limit of 100 MB")]
	HeaderTooLarge { size: usize },
}
