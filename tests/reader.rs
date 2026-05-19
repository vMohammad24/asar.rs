use asar::{AsarReader, AsarWriter};
use std::path::Path;

#[test]
fn test_read_nonexistent_file() {
	let mut writer = AsarWriter::new();
	writer.write_file("exists.txt", b"data", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert!(reader.read(Path::new("nope.txt")).is_none());
}

#[test]
fn test_read_dir_listing() {
	let mut writer = AsarWriter::new();
	writer.write_file("dir/a.txt", b"a", false).unwrap();
	writer.write_file("dir/b.txt", b"b", false).unwrap();
	writer.write_file("dir/c.txt", b"c", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let dir_contents = reader.read_dir(Path::new("dir")).unwrap();
	assert_eq!(dir_contents.len(), 3);
}

#[test]
fn test_read_dir_nonexistent() {
	let mut writer = AsarWriter::new();
	writer.write_file("file.txt", b"data", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert!(reader.read_dir(Path::new("nonexistent")).is_none());
}

#[test]
fn test_files_map_keys() {
	let mut writer = AsarWriter::new();
	writer.write_file("x.txt", b"x", false).unwrap();
	writer.write_file("y.txt", b"y", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let keys: Vec<_> = reader.files().keys().collect();
	assert!(keys.contains(&&std::path::PathBuf::from("x.txt")));
	assert!(keys.contains(&&std::path::PathBuf::from("y.txt")));
}

#[test]
fn test_directories_map_structure() {
	let mut writer = AsarWriter::new();
	writer.write_file("root.txt", b"r", false).unwrap();
	writer.write_file("sub/a.txt", b"a", false).unwrap();
	writer.write_file("sub/deep/b.txt", b"b", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();

	assert_eq!(reader.directories().len(), 3);
	assert!(reader.directories().contains_key(Path::new("")));
	assert!(reader.directories().contains_key(Path::new("sub")));
	assert!(reader.directories().contains_key(Path::new("sub/deep")));
}

#[test]
fn test_round_trip_preserves_data() {
	let original_data = b"The quick brown fox jumps over the lazy dog";

	let mut writer = AsarWriter::new();
	writer
		.write_file("message.txt", original_data, false)
		.unwrap();

	let mut buf1 = Vec::new();
	writer.finalize(&mut buf1).unwrap();

	let reader1 = AsarReader::new(&buf1, None).unwrap();
	let mut writer2 = AsarWriter::new();
	writer2
		.write_file(
			"message.txt",
			reader1.read(Path::new("message.txt")).unwrap().data(),
			false,
		)
		.unwrap();

	let mut buf2 = Vec::new();
	writer2.finalize(&mut buf2).unwrap();

	let reader2 = AsarReader::new(&buf2, None).unwrap();
	assert_eq!(
		reader2.read(Path::new("message.txt")).unwrap().data(),
		original_data
	);
}

#[test]
fn test_add_from_reader() {
	let mut writer1 = AsarWriter::new();
	writer1.write_file("a.txt", b"aaa", false).unwrap();
	writer1.write_file("b.txt", b"bbb", false).unwrap();

	let mut buf = Vec::new();
	writer1.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();

	let mut writer2 = AsarWriter::new();
	writer2.add_from_reader(&reader).unwrap();

	let mut buf2 = Vec::new();
	writer2.finalize(&mut buf2).unwrap();

	let reader2 = AsarReader::new(&buf2, None).unwrap();
	assert_eq!(reader2.files().len(), 2);
	assert_eq!(reader2.read(Path::new("a.txt")).unwrap().data(), b"aaa");
	assert_eq!(reader2.read(Path::new("b.txt")).unwrap().data(), b"bbb");
}

#[test]
fn test_reader_file_integrity_info_present() {
	let mut writer = AsarWriter::new();
	writer.write_file("file.txt", b"data", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("file.txt")).unwrap();
	assert!(file.integrity().is_some());
	let integrity = file.integrity().unwrap();
	assert_eq!(integrity.block_size(), 4 * 1024 * 1024);
}

#[test]
fn test_read_empty_archive() {
	let writer = AsarWriter::new();
	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert!(reader.files().is_empty());
	assert!(reader.symlinks().is_empty());
	assert!(reader.directories().is_empty());
}

#[test]
fn test_read_truncated_data() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("big.txt", b"some data here that will be truncated", false)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let truncated = &buf[..buf.len() / 2];
	let result = AsarReader::new(truncated, None);
	assert!(result.is_err());
}
