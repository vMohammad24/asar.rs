use crate::{
	error::{Error, Result},
	header::{FileIntegrity, FileLocation, Header},
};
use std::{
	borrow::Cow,
	collections::BTreeMap,
	path::{Path, PathBuf},
};

/// A reader for asar archives.
#[derive(Debug, Clone, PartialEq)]
pub struct AsarReader<'a> {
	header: Header,
	directories: BTreeMap<PathBuf, Vec<PathBuf>>,
	files: BTreeMap<PathBuf, AsarFile<'a>>,
	symlinks: BTreeMap<PathBuf, PathBuf>,
	verify_integrity: bool,
}

impl<'a> AsarReader<'a> {
	/// Creates a new AsarReader from the given data.
	pub fn new(data: &'a [u8], asar_path: Option<&Path>) -> Result<Self> {
		let (header, offset) = Header::read(&mut &data[..])?;
		Self::build(header, offset, data, asar_path, false)
	}

	/// Creates a new AsarReader from the given data and enables integrity checking.
	pub fn new_with_integrity_check(data: &'a [u8], asar_path: Option<&Path>) -> Result<Self> {
		let (header, offset) = Header::read(&mut &data[..])?;
		Self::build(header, offset, data, asar_path, true)
	}

	/// Creates a new AsarReader from an already parsed header.
	pub fn new_from_header(
		header: Header,
		offset: usize,
		data: &'a [u8],
		asar_path: Option<&Path>,
		verify_integrity: bool,
	) -> Result<Self> {
		Self::build(header, offset, data, asar_path, verify_integrity)
	}

	fn build(
		header: Header,
		offset: usize,
		data: &'a [u8],
		asar_path: Option<&Path>,
		verify_integrity: bool,
	) -> Result<Self> {
		let unpacked_dirs = asar_path.map(discover_unpacked_dirs);

		let mut files = BTreeMap::new();
		let mut directories = BTreeMap::new();
		let mut symlinks = BTreeMap::new();
		let mut context = ReadContext {
			file_map: &mut files,
			dir_map: &mut directories,
			symlink_map: &mut symlinks,
			begin_offset: offset,
			data,
			unpacked_dirs: unpacked_dirs.as_ref(),
			verify_integrity,
		};
		recursive_read(PathBuf::new(), &header, &mut context)?;
		Ok(Self {
			header,
			files,
			directories,
			symlinks,
			verify_integrity,
		})
	}

	/// Returns a map of all files in the archive.
	#[inline]
	pub const fn files(&self) -> &BTreeMap<PathBuf, AsarFile<'a>> {
		&self.files
	}

	/// Returns a map of all directories in the archive.
	#[inline]
	pub const fn directories(&self) -> &BTreeMap<PathBuf, Vec<PathBuf>> {
		&self.directories
	}

	/// Returns a map of all symlinks in the archive.
	#[inline]
	pub const fn symlinks(&self) -> &BTreeMap<PathBuf, PathBuf> {
		&self.symlinks
	}

	/// Reads a file from the archive.
	#[inline]
	pub fn read(&self, path: &Path) -> Option<&AsarFile<'_>> {
		if let Some(link) = self.symlinks.get(path) {
			return self.files.get(link);
		}
		self.files.get(path)
	}

	/// Reads a directory's contents from the archive.
	#[inline]
	pub fn read_dir(&self, path: &Path) -> Option<&[PathBuf]> {
		self.directories.get(path).map(|paths| paths.as_slice())
	}
}

/// A file in an asar archive.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AsarFile<'a> {
	data: Cow<'a, [u8]>,
	integrity: Option<FileIntegrity>,
}

impl<'a> AsarFile<'a> {
	/// Returns the data of the file.
	#[inline]
	pub fn data(&self) -> &[u8] {
		self.data.as_ref()
	}

	/// Returns the integrity information of the file.
	#[inline]
	pub const fn integrity(&self) -> Option<&FileIntegrity> {
		self.integrity.as_ref()
	}
}

struct ReadContext<'a, 'b> {
	file_map: &'b mut BTreeMap<PathBuf, AsarFile<'a>>,
	dir_map: &'b mut BTreeMap<PathBuf, Vec<PathBuf>>,
	symlink_map: &'b mut BTreeMap<PathBuf, PathBuf>,
	begin_offset: usize,
	data: &'a [u8],
	unpacked_dirs: Option<&'b Vec<PathBuf>>,
	verify_integrity: bool,
}

fn recursive_read<'a>(path: PathBuf, header: &Header, ctx: &mut ReadContext<'a, '_>) -> Result<()> {
	match header {
		Header::File(file) => {
			let file_data = match file.location() {
				FileLocation::Offset { offset } => {
					let start = ctx.begin_offset + offset;
					let end = start + file.size();
					if ctx.data.len() < end {
						return Err(Error::Truncated);
					}
					Cow::Borrowed(&ctx.data[start..end])
				}
				FileLocation::Unpacked { .. } => match ctx.unpacked_dirs {
					None => Cow::Borrowed([].as_slice()),
					Some(candidates) => {
						let mut found_file_data = None;
						for unpacked_dir in candidates {
							if !unpacked_dir.exists() {
								continue;
							}
							let file_path = unpacked_dir.join(&path);
							if file_path.exists()
								&& let Ok(data) = std::fs::read(&file_path)
							{
								found_file_data = Some(Cow::Owned(data));
								break;
							}
							let normalized_path = path
								.to_string_lossy()
								.replace("/", std::path::MAIN_SEPARATOR_STR);
							let alt_file_path = unpacked_dir.join(normalized_path);
							if alt_file_path.exists()
								&& let Ok(data) = std::fs::read(&alt_file_path)
							{
								found_file_data = Some(Cow::Owned(data));
								break;
							}
						}
						found_file_data.ok_or_else(|| Error::UnpackedFileNotFound(path.clone()))?
					}
				},
			};

			if ctx.verify_integrity
				&& let Some(integrity) = file.integrity()
			{
				let should_verify = match file.location() {
					FileLocation::Unpacked { .. } => !file_data.is_empty(),
					_ => true,
				};

				if should_verify {
					let algorithm = integrity.algorithm();
					let block_size = integrity.block_size();
					let blocks = integrity.blocks();

					if block_size > 0 && !blocks.is_empty() {
						for (idx, (block, expected_hash)) in
							file_data.chunks(block_size).zip(blocks.iter()).enumerate()
						{
							let hash = algorithm.hash(block);
							if hash != *expected_hash {
								return Err(Error::HashMismatch {
									file: path.clone(),
									block: Some(idx + 1),
									expected: expected_hash.clone(),
									actual: hash,
								});
							}
						}
					}

					let hash = algorithm.hash(&file_data);
					if hash != integrity.hash() {
						return Err(Error::HashMismatch {
							file: path.clone(),
							block: None,
							expected: integrity.hash().to_owned(),
							actual: hash,
						});
					}
				}
			}

			ctx.file_map.insert(
				path,
				AsarFile {
					data: file_data,
					integrity: file.integrity().cloned(),
				},
			);
		}
		Header::Directory { files } => {
			for (name, header) in files {
				let file_path = path.join(name);
				ctx.dir_map
					.entry(path.clone())
					.or_default()
					.push(file_path.clone());
				recursive_read(file_path, header, ctx)?;
			}
		}
		Header::Link { link } => {
			ctx.symlink_map.insert(path, link.clone());
		}
	}
	Ok(())
}

fn discover_unpacked_dirs(asar_path: &Path) -> Vec<PathBuf> {
	let asar_dir = asar_path.parent().unwrap_or(asar_path);
	let mut candidates = vec![
		asar_path.with_extension("asar.unpacked"),
		asar_dir.join("app.asar.unpacked"),
		asar_dir.join("original.asar.unpacked"),
		asar_dir.join("default_app.asar.unpacked"),
	];
	if let Ok(entries) = std::fs::read_dir(asar_dir) {
		for entry in entries.flatten() {
			if let Some(name) = entry.file_name().to_str()
				&& name.ends_with(".asar.unpacked")
				&& entry.path().is_dir()
			{
				candidates.push(entry.path());
			}
		}
	}
	candidates
}
