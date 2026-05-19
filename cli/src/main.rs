use asar::{AsarReader, AsarWriter};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "asar")]
#[command(version = "0.1.0")]
#[command(about = "ASAR archive management CLI", long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand)]
enum Commands {
	/// Create asar archive
	#[command(alias = "p")]
	Pack {
		/// Source directory
		dir: PathBuf,
		/// Output asar file
		output: PathBuf,
	},
	/// List files of asar archive
	#[command(alias = "l")]
	List {
		/// Asar archive path
		archive: PathBuf,
	},
	/// Extract one file from archive
	#[command(alias = "ef")]
	ExtractFile {
		/// Asar archive path
		archive: PathBuf,
		/// Filename to extract
		filename: String,
	},
	/// Extract archive
	#[command(alias = "e")]
	Extract {
		/// Asar archive path
		archive: PathBuf,
		/// Destination directory
		dest: PathBuf,
	},
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = Cli::parse();

	match cli.command {
		Commands::Pack { dir, output } => {
			let mut writer = AsarWriter::new();
			let root = Path::new(&dir);

			for entry in WalkDir::new(root).follow_links(false) {
				let entry = entry?;
				let path = entry.path();
				let relative_path = path.strip_prefix(root)?;

				if entry.file_type().is_file() {
					let bytes = fs::read(path)?;
					let executable = {
						#[cfg(unix)]
						{
							use std::os::unix::fs::PermissionsExt;
							entry.metadata()?.permissions().mode() & 0o111 != 0
						}
						#[cfg(not(unix))]
						false
					};
					writer.write_file(relative_path, bytes, executable)?;
				} else if entry.file_type().is_symlink() {
					let target = fs::read_link(path)?;
					writer.write_symlink(relative_path, target)?;
				}
			}

			let mut out_file = fs::File::create(output)?;
			writer.finalize(&mut out_file)?;
			println!("Archive created successfully.");
		}
		Commands::List { archive } => {
			let data = fs::read(&archive)?;
			let reader = AsarReader::new(&data, Some(&archive))?;
			for path in reader.files().keys() {
				println!("{}", path.display());
			}
		}
		Commands::ExtractFile { archive, filename } => {
			let data = fs::read(&archive)?;
			let reader = AsarReader::new(&data, Some(&archive))?;
			let path = Path::new(&filename);

			if let Some(file) = reader.read(path) {
				fs::write(path.file_name().unwrap_or(path.as_os_str()), file.data())?;
				println!("File {} extracted.", filename);
			} else {
				eprintln!("File {} not found in archive.", filename);
				std::process::exit(1);
			}
		}
		Commands::Extract { archive, dest } => {
			let data = fs::read(&archive)?;
			let reader = AsarReader::new(&data, Some(&archive))?;

			fs::create_dir_all(&dest)?;

			for (path, file) in reader.files() {
				let safe_path: PathBuf = path
					.components()
					.filter(|c| matches!(c, std::path::Component::Normal(_)))
					.collect();
				if safe_path.as_os_str().is_empty() {
					continue;
				}
				let target_path = dest.join(safe_path);
				if let Some(parent) = target_path.parent() {
					fs::create_dir_all(parent)?;
				}
				fs::write(target_path, file.data())?;
			}
			println!("Archive extracted to {}.", dest.display());
		}
	}

	Ok(())
}
