# ASAR(.rs)

A fast, memory-safe Rust library and CLI tool for reading, writing, and managing ASAR archives.

This project consists of two parts:
* `asar`: library for archive manipulation.
* `asar-cli`: A CLI for packing, extracting, and inspecting archives without any hassale.

---

## Features

- **Read & Write:** Full support for parsing and generating `.asar` files.
- **Integrity Checking:** Built-in SHA256 integrity validation for archive files.
- **Symlink Support:** Preserves and resolves symbolic links within the archive.
- **Unpacked Files:** Support for reading external `.asar.unpacked` files seamlessly.
- **Executable Flags:** Retains Unix executable permissions when packing and unpacking.
- **Pure Rust:** Fast, safe, and heavily tested.

---

## Installation


### NixOS
If you're using NixOS, you can install the `asar` CLI directly using the flake:

```bash
nix profile add "github:vMohammad24/asar.rs"
```

### From source
```bash
Ensure you have Rust and Cargo installed. To build and install the CLI from the source:

```bash
git clone https://github.com/vMohammad24/asar.rs.git
cd asar.rs
cargo install --path cli

```

To use the library in your Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
asar = { git = "https://github.com/vMohammad24/asar.rs.git" }

```

---

## CLI

The `asar` CLI provides a number of commands for managing archives. You can use full command names or their convenient aliases similarily to the official ASAR CLI.

### Pack a Directory

Pack a directory into an `.asar` archive.

```bash
asar (p)ack <source-directory> <output.asar>
```

### Extract an Archive

Extract an entire `.asar` archive to a specified destination directory.

```bash
asar (e)xtract <archive.asar> <destination-directory>
```

### Extract a Single File

Extract a specific file from an archive into your current directory.

```bash
asar extract-file(ef) <archive.asar> <filename/inside/archive.txt>
```

### List Files

List all file paths contained within an `.asar` archive.

```bash
asar (l)ist <archive.asar>
```

---

## Library

The `asar` crate exposes an advanced API for reading and writing archives easily.

### Reading an Archive

```rust
use asar::AsarReader;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let archive_path = "app.asar";
    let data = fs::read(archive_path)?;
    
    // Initialize the reader (optionally pass the path to resolve .unpacked files)
    let reader = AsarReader::new(&data, Some(archive_path.into()))?;

    // Read a specific file
    if let Some(file) = reader.read(Path::new("package.json")) {
        println!("File size: {} bytes", file.data().len());
        println!("Content:\n{}", String::from_utf8_lossy(file.data()));
    }

    // List directory contents
    if let Some(contents) = reader.read_dir(Path::new("src")) {
        for path in contents {
            println!("Found: {}", path.display());
        }
    }

    Ok(())
}

```

### Writing an Archive

```rust
use asar::AsarWriter;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = AsarWriter::new();

    // Add files to the archive (path, data, is_executable)
    writer.write_file("hello.txt", b"Hello, World!", false)?;
    writer.write_file("bin/run.sh", b"#!/bin/bash\necho 'running'", true)?;

    // Create the output file and finalize the archive
    let mut out_file = File::create("output.asar")?;
    writer.finalize(&mut out_file)?;

    println!("Archive built successfully!");
    Ok(())
}

```

### Integrity Checking

You can strictly validate file hashes by initializing the reader with integrity checking enabled:

```rust
use asar::AsarReader;

let data = std::fs::read("app.asar").unwrap();
// This will throw an error if file hashes don't match the header
let reader = AsarReader::new_with_integrity_check(&data, None).unwrap();

```

---

## LICENSE

This project is licensed under the [AGPLv3](./LICENSE).
