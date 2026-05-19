use asar::{AsarReader, AsarWriter, Error};
use std::path::Path;

#[test]
fn test_empty_file_data() {
	let mut writer = AsarWriter::new();
	writer.write_file("empty", b"", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.read(Path::new("empty")).unwrap().data(), b"");
}

#[test]
fn test_duplicate_file_error() {
	let mut writer = AsarWriter::new();
	writer.write_file("dup.txt", b"first", false).unwrap();
	let err = writer.write_file("dup.txt", b"second", false).unwrap_err();
	assert!(matches!(err, Error::FileAlreadyWritten(p) if p == Path::new("dup.txt").to_path_buf()));
}

#[test]
fn test_truncated_archive() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("data.bin", b"some important data", false)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let result = AsarReader::new(&buf[..20], None);
	assert!(result.is_err());
}

#[test]
fn test_invalid_path_with_parent_dir() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("../escape.txt", b"escape", false)
		.unwrap();
	let mut buf = Vec::new();
	let result = writer.finalize(&mut buf);
	assert!(result.is_err());
}

#[test]
fn test_invalid_path_with_current_dir() {
	let mut writer = AsarWriter::new();
	writer.write_file("./file.txt", b"data", false).unwrap();
	let mut buf = Vec::new();
	let result = writer.finalize(&mut buf);
	assert!(result.is_err());
}

#[test]
fn test_unicode_filenames() {
	let mut writer = AsarWriter::new();
	writer.write_file("日本語.txt", b"japanese", false).unwrap();
	writer.write_file("emoji🎉.txt", b"emoji", false).unwrap();
	writer
		.write_file("Café/naïve.txt", b"accents", false)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(
		reader.read(Path::new("日本語.txt")).unwrap().data(),
		b"japanese"
	);
	assert_eq!(
		reader.read(Path::new("emoji🎉.txt")).unwrap().data(),
		b"emoji"
	);
	assert_eq!(
		reader.read(Path::new("Café/naïve.txt")).unwrap().data(),
		b"accents"
	);
}

#[test]
fn test_spaces_in_filenames() {
	let mut writer = AsarWriter::new();
	writer.write_file("my file.txt", b"spaced", false).unwrap();
	writer
		.write_file("my dir/content.txt", b"nested spaced", false)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(
		reader.read(Path::new("my file.txt")).unwrap().data(),
		b"spaced"
	);
	assert_eq!(
		reader.read(Path::new("my dir/content.txt")).unwrap().data(),
		b"nested spaced"
	);
}

#[test]
fn test_special_characters_in_data() {
	let data = b"\x00\x01\x02\xff\xfe\xfd\x80\x90\xa0\xb0\xc0\xd0\xe0\xf0";
	let mut writer = AsarWriter::new();
	writer.write_file("special.bin", data, false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(reader.read(Path::new("special.bin")).unwrap().data(), data);
}

#[test]
fn test_very_long_filename() {
	let long_name = "a".repeat(255) + ".txt";
	let mut writer = AsarWriter::new();
	writer.write_file(&long_name, b"long name", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new(&long_name)).unwrap();
	assert_eq!(file.data(), b"long name");
}

#[test]
fn test_write_file_as_path_buf() {
	let mut writer = AsarWriter::new();
	let path = std::path::PathBuf::from("pathbuf.txt");
	writer.write_file(&path, b"pathbuf data", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert_eq!(
		reader.read(Path::new("pathbuf.txt")).unwrap().data(),
		b"pathbuf data"
	);
}

#[test]
fn test_finalized_bytes_are_valid() {
	let mut writer = AsarWriter::new();
	writer.write_file("test.txt", b"content", false).unwrap();

	let mut buf = Vec::new();
	let written = writer.finalize(&mut buf).unwrap();
	assert_eq!(written, buf.len());
	assert!(buf.len() > 16);
}

#[test]
fn test_writer_with_algorithm_custom() {
	let writer = AsarWriter::new_with_algorithm(asar::header::HashAlgorithm::Sha256);
	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	assert!(reader.files().is_empty());
}

#[test]
fn test_multiple_archives_independent() {
	let mut writer1 = AsarWriter::new();
	writer1.write_file("file.txt", b"archive1", false).unwrap();

	let mut writer2 = AsarWriter::new();
	writer2.write_file("file.txt", b"archive2", false).unwrap();

	let mut buf1 = Vec::new();
	let mut buf2 = Vec::new();
	writer1.finalize(&mut buf1).unwrap();
	writer2.finalize(&mut buf2).unwrap();

	let reader1 = AsarReader::new(&buf1, None).unwrap();
	let reader2 = AsarReader::new(&buf2, None).unwrap();

	assert_eq!(
		reader1.read(Path::new("file.txt")).unwrap().data(),
		b"archive1"
	);
	assert_eq!(
		reader2.read(Path::new("file.txt")).unwrap().data(),
		b"archive2"
	);
}
