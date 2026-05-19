//! asar.rs is a library for reading and writing ASAR archives.
//!
//! the file format ASAR (Atom Shell Archive) is a simple archive format,
//! mainly used in electron applications.

pub mod error;
pub mod header;
pub mod integrity;
pub mod reader;
pub mod writer;

pub use error::{Error, Result};
pub use reader::{AsarFile, AsarReader};
pub use writer::AsarWriter;
