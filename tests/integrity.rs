use asar::header::HashAlgorithm;
use asar::integrity::IntegrityChecker;
use asar::{AsarReader, AsarWriter};
use std::path::Path;

#[test]
fn test_integrity_check_passes() {
	let mut writer = AsarWriter::new();
	writer.write_file("file.txt", b"verify me", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new_with_integrity_check(&buf, None).unwrap();
	let file = reader.read(Path::new("file.txt")).unwrap();
	assert_eq!(file.data(), b"verify me");
}

#[test]
fn test_integrity_check_large_file() {
	let data = vec![0x42_u8; 6 * 1024 * 1024];
	let mut writer = AsarWriter::new();
	writer.write_file("big.bin", &data, false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new_with_integrity_check(&buf, None).unwrap();
	let file = reader.read(Path::new("big.bin")).unwrap();
	assert_eq!(file.data(), &data[..]);
}

#[test]
fn test_integrity_check_multiple_files() {
	let mut writer = AsarWriter::new();
	writer.write_file("a.txt", b"file a", false).unwrap();
	writer.write_file("b.txt", b"file b", false).unwrap();
	writer.write_file("dir/c.txt", b"file c", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new_with_integrity_check(&buf, None).unwrap();
	assert_eq!(reader.files().len(), 3);
	assert_eq!(reader.read(Path::new("a.txt")).unwrap().data(), b"file a");
	assert_eq!(reader.read(Path::new("b.txt")).unwrap().data(), b"file b");
	assert_eq!(
		reader.read(Path::new("dir/c.txt")).unwrap().data(),
		b"file c"
	);
}

#[test]
fn test_hash_algorithm_sha256() {
	let hash = HashAlgorithm::Sha256.hash(b"test");
	assert_eq!(hash.len(), 32);
}

#[test]
fn test_hash_algorithm_hash_blocks() {
	let algo = HashAlgorithm::Sha256;
	let data = vec![0u8; 100];
	let blocks = algo.hash_blocks(32, &data);
	assert_eq!(blocks.len(), 4);

	let data2 = vec![0u8; 32];
	let blocks2 = algo.hash_blocks(32, &data2);
	assert_eq!(blocks2.len(), 1);
}

#[test]
fn test_integrity_checker_verify_file() {
	let checker = IntegrityChecker::default();
	let data = b"hello world";
	let hash = HashAlgorithm::Sha256.hash(data);

	assert!(checker.verify_file(data, &hash));
	assert!(!checker.verify_file(b"wrong data", &hash));
}

#[test]
fn test_integrity_checker_verify_blocks() {
	let checker = IntegrityChecker::new(HashAlgorithm::Sha256);
	let data = vec![1u8; 100];
	let blocks = HashAlgorithm::Sha256.hash_blocks(32, &data);

	assert!(checker.verify_blocks(&data, 32, &blocks));
	assert!(!checker.verify_blocks(&[0u8; 100], 32, &blocks));
}

#[test]
fn test_integrity_check_detects_corruption() {
	let mut writer = AsarWriter::new();
	writer
		.write_file("file.txt", b"original data", false)
		.unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let json_size = u32::from_le_bytes(buf[12..16].try_into().unwrap()) as usize;
	let header_end = 16 + ((json_size + 3) & !3);
	if header_end < buf.len() {
		buf[header_end] ^= 0xFF;
	}

	let result = AsarReader::new_with_integrity_check(&buf, None);
	assert!(result.is_err());
}

#[test]
fn test_file_integrity_has_block_info() {
	let mut writer = AsarWriter::new();
	writer.write_file("file.txt", b"data", false).unwrap();

	let mut buf = Vec::new();
	writer.finalize(&mut buf).unwrap();

	let reader = AsarReader::new(&buf, None).unwrap();
	let file = reader.read(Path::new("file.txt")).unwrap();
	let integrity = file.integrity().unwrap();
	assert!(!integrity.hash().is_empty());
	assert!(integrity.block_size() > 0);
}

#[test]
fn test_hash_algorithm_display() {
	assert_eq!(format!("{}", HashAlgorithm::Sha256), "SHA256");
}

#[test]
fn test_hash_algorithm_from_str() {
	assert!(matches!(
		"SHA256".parse::<HashAlgorithm>(),
		Ok(HashAlgorithm::Sha256)
	));
	assert!(matches!(
		"sha256".parse::<HashAlgorithm>(),
		Ok(HashAlgorithm::Sha256)
	));
	assert!(matches!(
		"sha-256".parse::<HashAlgorithm>(),
		Ok(HashAlgorithm::Sha256)
	));
	assert!("unknown".parse::<HashAlgorithm>().is_err());
}
