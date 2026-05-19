use asar::{AsarReader, AsarWriter};
use std::path::Path;

#[test]
fn test_write_single_file() {
	let mut writer = AsarWriter::new();
	writer.write_file("hello.txt", b"hello", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("hello.txt")).unwrap();
	assert_eq!(file.data(), b"hello");
}

#[test]
fn test_write_empty_file() {
	let mut writer = AsarWriter::new();
	writer.write_file("empty.txt", b"", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("empty.txt")).unwrap();
	assert_eq!(file.data(), b"");
}

#[test]
fn test_write_multiple_files_same_dir() {
	let mut writer = AsarWriter::new();
	writer.write_file("a.txt", b"aaa", false).unwrap();
	writer.write_file("b.txt", b"bbb", false).unwrap();
	writer.write_file("c.txt", b"ccc", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.read(Path::new("a.txt")).unwrap().data(), b"aaa");
	assert_eq!(reader.read(Path::new("b.txt")).unwrap().data(), b"bbb");
	assert_eq!(reader.read(Path::new("c.txt")).unwrap().data(), b"ccc");
	assert_eq!(reader.files().len(), 3);
}

#[test]
fn test_write_deeply_nested_files() {
	let mut writer = AsarWriter::new();
	writer.write_file("a/b/c/d/e.txt", b"deep", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("a/b/c/d/e.txt")).unwrap();
	assert_eq!(file.data(), b"deep");

	assert!(reader.directories().contains_key(Path::new("a")));
	assert!(reader.directories().contains_key(Path::new("a/b")));
	assert!(reader.directories().contains_key(Path::new("a/b/c")));
	assert!(reader.directories().contains_key(Path::new("a/b/c/d")));
}

#[test]
fn test_write_duplicate_file_errors() {
	let mut writer = AsarWriter::new();
	writer.write_file("same.txt", b"first", false).unwrap();
	let result = writer.write_file("same.txt", b"second", false);
	assert!(result.is_err());
}

#[test]
fn test_write_binary_data() {
	let data: Vec<u8> = (0..=255).collect();
	let mut writer = AsarWriter::new();
	writer.write_file("binary.bin", &data, false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("binary.bin")).unwrap();
	assert_eq!(file.data(), &data[..]);
}

#[test]
fn test_write_large_file_multiple_blocks() {
	let data = vec![0xAB_u8; 5 * 1024 * 1024];
	let mut writer = AsarWriter::new();
	writer.write_file("large.bin", &data, false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new_with_integrity_check(&buf, None).unwrap();
	let file = reader.read(Path::new("large.bin")).unwrap();
	assert_eq!(file.data().len(), data.len());
	assert_eq!(file.data(), &data[..]);
}

#[test]
fn test_write_executable_file() {
	let mut writer = AsarWriter::new();
	writer.write_file("script.sh", b"#!/bin/sh", true).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("script.sh")).unwrap();
	assert_eq!(file.data(), b"#!/bin/sh");
}

#[test]
fn test_finalize_empty_archive() {
	let writer = AsarWriter::new();
	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert!(reader.files().is_empty());
	assert!(reader.directories().is_empty());
	assert!(reader.symlinks().is_empty());
}

#[test]
fn test_write_multiple_directories_with_files() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("src/main.rs", b"fn main() {}", false)
		.unwrap();
	writer
		.write_file("src/lib.rs", b"pub fn lib() {}", false)
		.unwrap();
	writer
		.write_file("tests/test.rs", b"#[test] fn test() {}", false)
		.unwrap();
	writer.write_file("README.md", b"# Project", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.files().len(), 4);
	assert_eq!(
		reader.read(Path::new("src/main.rs")).unwrap().data(),
		b"fn main() {}"
	);
	assert_eq!(
		reader.read(Path::new("tests/test.rs")).unwrap().data(),
		b"#[test] fn test() {}"
	);
}

#[test]
fn test_write_with_path_objects() {
	let mut writer = AsarWriter::new();
	writer
		.write_file(
			std::path::Path::new("dir").join("file.txt"),
			b"path obj",
			false,
		)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(
		reader.read(Path::new("dir/file.txt")).unwrap().data(),
		b"path obj"
	);
}
