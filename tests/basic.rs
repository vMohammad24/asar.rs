use asar::{AsarReader, AsarWriter};
use std::path::Path;

#[test]
fn test_basic_write_read() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("file1.txt", b"hello world", false)
		.unwrap();
	writer
		.write_file("dir/file2.txt", b"nested file", false)
		.unwrap();

	let mut buffer = Vec::new();
	writer.finalize(&mut buffer).unwrap();

	let reader = AsarReader::new(&buffer, None).unwrap();

	let file1 = reader.read(Path::new("file1.txt")).unwrap();
	assert_eq!(file1.data(), b"hello world");

	let file2 = reader.read(Path::new("dir/file2.txt")).unwrap();
	assert_eq!(file2.data(), b"nested file");
}

#[test]
fn test_integrity() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("file1.txt", b"hello world", false)
		.unwrap();

	let mut buffer = Vec::new();
	writer.finalize(&mut buffer).unwrap();

	let reader = AsarReader::new_with_integrity_check(&buffer, None).unwrap();

	let file1 = reader.read(Path::new("file1.txt")).unwrap();
	assert_eq!(file1.data(), b"hello world");
}
