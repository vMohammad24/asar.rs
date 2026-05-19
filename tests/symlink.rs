use asar::{AsarReader, AsarWriter};
use std::path::Path;

#[test]
fn test_write_symlink() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("real.txt", b"real content", false)
		.unwrap();
	writer
		.write_symlink("link.txt", Path::new("real.txt"))
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.symlinks().len(), 1);
	assert_eq!(
		reader.symlinks().get(Path::new("link.txt")),
		Some(&std::path::PathBuf::from("real.txt"))
	);
}

#[test]
fn test_read_symlink_resolves() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("target.txt", b"target data", false)
		.unwrap();
	writer
		.write_symlink("shortcut.txt", Path::new("target.txt"))
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("shortcut.txt")).unwrap();
	assert_eq!(file.data(), b"target data");
}

#[test]
fn test_symlink_in_directory() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("dir/original.txt", b"original", false)
		.unwrap();
	writer
		.write_symlink("dir/link.txt", Path::new("dir/original.txt"))
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("dir/link.txt")).unwrap();
	assert_eq!(file.data(), b"original");
}

#[test]
fn test_multiple_symlinks_to_same_target() {
	let mut writer = AsarWriter::new();
	writer.write_file("real.txt", b"data", false).unwrap();
	writer
		.write_symlink("link1.txt", Path::new("real.txt"))
		.unwrap();
	writer
		.write_symlink("link2.txt", Path::new("real.txt"))
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.symlinks().len(), 2);
	assert_eq!(reader.read(Path::new("link1.txt")).unwrap().data(), b"data");
	assert_eq!(reader.read(Path::new("link2.txt")).unwrap().data(), b"data");
}

#[test]
fn test_symlink_and_files_coexist() {
	let mut writer = AsarWriter::new();
	writer.write_file("file.txt", b"file data", false).unwrap();
	writer
		.write_symlink("slink", Path::new("file.txt"))
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.files().len(), 1);
	assert_eq!(reader.symlinks().len(), 1);
}
