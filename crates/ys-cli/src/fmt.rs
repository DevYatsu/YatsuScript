//! # ys-cli fmt
//!
//! A simple ysc code formatter.

use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

/// Format all ysc files in a directory or a single file.
pub fn format_all(input: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut files = Vec::<PathBuf>::new();

    if input.is_file() {
        if input.extension().is_some_and(|ext| ext == "ys") {
            files.push(input.to_path_buf());
        }
    } else {
        for entry in fs::read_dir(input)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "ys") {
                files.push(path);
            }
        }
    }

    for path in files {
        let source = fs::read_to_string(&path)?;
        let formatted = ys_core::fmt::format_source(&source);
        fs::write(&path, formatted)?;
        println!("{} {}", "Formatted".green(), path.display());
    }

    Ok(())
}
