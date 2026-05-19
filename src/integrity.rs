use crate::header::HashAlgorithm;

/// A utility for verifying the integrity of files in an asar archive.
pub struct IntegrityChecker {
	algorithm: HashAlgorithm,
}

impl IntegrityChecker {
	/// Creates a new IntegrityChecker.
	pub const fn new(algorithm: HashAlgorithm) -> Self {
		Self { algorithm }
	}

	pub fn verify_file(&self, data: &[u8], expected_hash: &[u8]) -> bool {
		let actual_hash = self.algorithm.hash(data);
		actual_hash == expected_hash
	}

	pub fn verify_blocks(
		&self,
		data: &[u8],
		block_size: usize,
		expected_blocks: &[Vec<u8>],
	) -> bool {
		let actual_blocks = self.algorithm.hash_blocks(block_size, data);
		actual_blocks == expected_blocks
	}
}

impl Default for IntegrityChecker {
	fn default() -> Self {
		Self::new(HashAlgorithm::Sha256)
	}
}
