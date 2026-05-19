use crate::{
	error::{Error, Result},
	header::{File, FileIntegrity, FileLocation, HashAlgorithm, Header},
	reader::AsarReader,
};
use byteorder::{LittleEndian, WriteBytesExt};
use std::{
	collections::{BTreeMap, VecDeque},
	io::Write,
	path::{Component, Path, PathBuf},
};

const BLOCK_SIZE: usize = 4 * 1024 * 1024;

/// A writer for creating asar archives.
pub struct AsarWriter {
	files: BTreeMap<PathBuf, File>,
	symlinks: BTreeMap<PathBuf, PathBuf>,
	buffer: Vec<u8>,
	offset: usize,
	hasher: HashAlgorithm,
}

impl AsarWriter {
	/// Creates a new AsarWriter with default settings.
	pub fn new() -> Self {
		Self::default()
	}

	/// Creates a new AsarWriter with a specific hash algorithm.
	pub const fn new_with_algorithm(hasher: HashAlgorithm) -> Self {
		Self {
			files: BTreeMap::new(),
			symlinks: BTreeMap::new(),
			buffer: Vec::new(),
			offset: 0,
			hasher,
		}
	}

	/// Adds files from an existing reader to this writer.
	pub fn add_from_reader(&mut self, reader: &AsarReader) -> Result<()> {
		for (path, file) in reader.files() {
			self.write_file(path, file.data(), false)?;
		}
		Ok(())
	}

	/// Writes a file to the archive.
	pub fn write_file(
		&mut self,
		path: impl AsRef<Path>,
		bytes: impl AsRef<[u8]>,
		executable: bool,
	) -> Result<()> {
		self.write_file_impl(path.as_ref(), bytes.as_ref(), executable)
	}

	/// Writes a symlink to the archive.
	pub fn write_symlink(&mut self, path: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
		self.symlinks
			.insert(path.as_ref().to_path_buf(), link.as_ref().to_path_buf());
		Ok(())
	}

	fn write_file_impl(&mut self, path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
		if self.files.contains_key(path) {
			return Err(Error::FileAlreadyWritten(path.to_path_buf()));
		}
		let file = File::new(
			FileLocation::Offset {
				offset: self.offset,
			},
			bytes.len(),
			executable,
			Some(FileIntegrity::new(
				self.hasher,
				self.hasher.hash(bytes),
				BLOCK_SIZE,
				self.hasher.hash_blocks(BLOCK_SIZE, bytes),
			)),
		);
		self.buffer.extend_from_slice(bytes);
		self.offset += bytes.len();
		self.files.insert(path.to_path_buf(), file);
		Ok(())
	}

	pub fn finalize<FinalWriter>(self, mut final_writer: FinalWriter) -> Result<usize>
	where
		FinalWriter: Write,
	{
		let mut header = Header::new();
		for (path, file) in self.files {
			let path = path_to_reverse_components(&path)?;
			recursive_add_to_header(path, Header::File(file), &mut header);
		}
		for (path, link) in self.symlinks {
			let path = path_to_reverse_components(&path)?;
			recursive_add_to_header(path, Header::Link { link }, &mut header);
		}
		let mut written = 0;
		let mut json = serde_json::to_string(&header)?.into_bytes();

		let json_size = json.len() as u32;
		let aligned_json_size = json_size + (4 - (json_size % 4)) % 4;
		json.resize(aligned_json_size as usize, 0);

		final_writer.write_u32::<LittleEndian>(4)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(aligned_json_size + 8)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(aligned_json_size + 4)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_u32::<LittleEndian>(json_size)?;
		written += std::mem::size_of::<u32>();
		final_writer.write_all(&json)?;
		written += json.len();
		final_writer.write_all(&self.buffer)?;
		written += self.buffer.len();
		final_writer.flush()?;
		Ok(written)
	}
}

impl Default for AsarWriter {
	fn default() -> Self {
		Self {
			files: BTreeMap::new(),
			symlinks: BTreeMap::new(),
			offset: 0,
			buffer: Vec::new(),
			hasher: HashAlgorithm::Sha256,
		}
	}
}

fn path_to_reverse_components(path: &Path) -> Result<VecDeque<String>> {
	let mut components = VecDeque::new();
	for component in path.components() {
		match component {
			Component::Prefix(_) | Component::RootDir => continue,
			Component::ParentDir | Component::CurDir => {
				return Err(Error::InvalidPath(path.to_path_buf()));
			}
			Component::Normal(p) => {
				components.push_back(p.to_string_lossy().into_owned());
			}
		}
	}
	if components.is_empty() {
		return Err(Error::InvalidPath(path.to_path_buf()));
	}
	Ok(components)
}

fn recursive_add_to_header(
	mut path: VecDeque<String>,
	file_or_symlink: Header,
	header: &mut Header,
) {
	let header_map = match header {
		Header::Directory { files } => files,
		_ => return,
	};
	if let Some(name) = path.pop_front() {
		if path.is_empty() {
			header_map.insert(name, file_or_symlink);
		} else {
			let new_header = header_map.entry(name).or_insert_with(Header::new);
			recursive_add_to_header(path, file_or_symlink, new_header);
		}
	}
}
