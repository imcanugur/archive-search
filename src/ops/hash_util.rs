use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn list_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(files)
}

pub fn files_identical(a: &Path, b: &Path) -> Result<bool, AppError> {
    if fs::metadata(a)?.len() != fs::metadata(b)?.len() {
        return Ok(false);
    }
    Ok(hash_file(a)? == hash_file(b)?)
}

fn hash_file(path: &Path) -> Result<blake3::Hash, AppError> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 32 * 1024];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        if let Some(slice) = buf.get(..read) {
            hasher.update(slice);
        }
    }
    Ok(hasher.finalize())
}

pub fn next_available_name(name: &str, occupied: &HashMap<String, PathBuf>) -> String {
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s, Some(e)),
        _ => (name, None),
    };
    for idx in 1_u32..100_000 {
        let candidate = match ext {
            Some(e) => format!("{stem}_{idx}.{e}"),
            None => format!("{stem}_{idx}"),
        };
        if !occupied.contains_key(&candidate.to_ascii_lowercase()) {
            return candidate;
        }
    }
    format!("{name}_dup")
}
