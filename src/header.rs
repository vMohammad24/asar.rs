use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, hex::Hex, serde_as};
use std::{
	collections::HashMap,
	fmt::{self, Display},
	path::PathBuf,
	str::FromStr,
};

/// The header of an asar archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Header {
	/// A file entry.
	File(File),
	/// A directory entry.
	Directory { files: HashMap<String, Self> },
	/// A symlink entry.
	Link { link: PathBuf },
}

impl Header {
	pub(crate) fn new() -> Self {
		Self::Directory {
			files: HashMap::new(),
		}
	}

	pub fn read<R: ReadBytesExt>(data: &mut R) -> Result<(Self, usize)> {
		const MAX_HEADER_SIZE: usize = 100 * 1024 * 1024;
		data.read_u32::<LittleEndian>()?;
		let header_size = data.read_u32::<LittleEndian>()? as usize;
		data.read_u32::<LittleEndian>()?;
		let json_size = data.read_u32::<LittleEndian>()? as usize;
		if json_size > MAX_HEADER_SIZE {
			return Err(Error::HeaderTooLarge { size: json_size });
		}
		let mut bytes = vec![0_u8; json_size];
		data.read_exact(&mut bytes)?;
		Ok((serde_json::from_slice(&bytes)?, header_size + 8))
	}
}

#[serde_as]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FileLocation {
	Offset {
		#[serde_as(as = "DisplayFromStr")]
		offset: usize,
	},

	Unpacked {
		#[serde(skip_serializing_if = "is_false")]
		unpacked: bool,
	},
}

impl FileLocation {
	/// Creates a new FileLocation::Offset.
	#[inline]
	pub const fn offset(offset: usize) -> Self {
		FileLocation::Offset { offset }
	}

	/// Creates a new FileLocation::Unpacked.
	#[inline]
	pub const fn unpacked() -> Self {
		FileLocation::Unpacked { unpacked: true }
	}
}

/// Information about a file in an asar archive.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct File {
	#[serde(flatten)]
	location: FileLocation,

	size: usize,

	#[serde(skip_serializing_if = "is_false", default = "default_false")]
	executable: bool,

	#[serde(skip_serializing_if = "Option::is_none")]
	integrity: Option<FileIntegrity>,
}

impl File {
	pub(crate) const fn new(
		location: FileLocation,
		size: usize,
		executable: bool,
		integrity: Option<FileIntegrity>,
	) -> Self {
		Self {
			location,
			size,
			executable,
			integrity,
		}
	}

	/// Returns the location of the file.
	#[inline]
	pub const fn location(&self) -> FileLocation {
		self.location
	}

	/// Returns the offset of the file if it is stored in the archive.
	#[inline]
	pub const fn offset(&self) -> Option<usize> {
		match self.location {
			FileLocation::Offset { offset } => Some(offset),
			_ => None,
		}
	}

	/// Returns true if the file is unpacked.
	#[inline]
	pub const fn unpacked(&self) -> bool {
		matches!(self.location, FileLocation::Unpacked { .. })
	}

	/// Returns the size of the file.
	#[inline]
	pub const fn size(&self) -> usize {
		self.size
	}

	/// Returns true if the file is executable.
	#[inline]
	pub const fn executable(&self) -> bool {
		self.executable
	}

	/// Returns the integrity information of the file.
	#[inline]
	pub const fn integrity(&self) -> Option<&FileIntegrity> {
		self.integrity.as_ref()
	}
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileIntegrity {
	algorithm: HashAlgorithm,

	#[serde_as(as = "Hex")]
	hash: Vec<u8>,

	block_size: usize,

	#[serde_as(as = "Vec<Hex>")]
	blocks: Vec<Vec<u8>>,
}

impl FileIntegrity {
	pub(crate) const fn new(
		algorithm: HashAlgorithm,
		hash: Vec<u8>,
		block_size: usize,
		blocks: Vec<Vec<u8>>,
	) -> Self {
		Self {
			algorithm,
			hash,
			block_size,
			blocks,
		}
	}

	#[inline]
	pub const fn algorithm(&self) -> HashAlgorithm {
		self.algorithm
	}

	#[inline]
	pub fn hash(&self) -> &[u8] {
		&self.hash
	}

	#[inline]
	pub const fn block_size(&self) -> usize {
		self.block_size
	}

	#[inline]
	pub fn blocks(&self) -> &[Vec<u8>] {
		&self.blocks
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum HashAlgorithm {
	#[serde(rename = "SHA256")]
	Sha256,
}

impl HashAlgorithm {
	pub fn hash(self, data: &[u8]) -> Vec<u8> {
		use sha2::{Digest, Sha256};
		match self {
			Self::Sha256 => Sha256::digest(data).to_vec(),
		}
	}

	pub fn hash_blocks(self, block_size: usize, data: &[u8]) -> Vec<Vec<u8>> {
		data.chunks(block_size)
			.map(|chunk| self.hash(chunk))
			.collect()
	}
}

impl Display for HashAlgorithm {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Sha256 => write!(f, "SHA256"),
		}
	}
}

impl FromStr for HashAlgorithm {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s.trim().to_lowercase().as_str() {
			"sha256" | "sha-256" => Ok(Self::Sha256),
			_ => Err(Error::InvalidHashAlgorithm(s.to_string())),
		}
	}
}

const fn is_false(b: &bool) -> bool {
	!*b
}

const fn default_false() -> bool {
	false
}
