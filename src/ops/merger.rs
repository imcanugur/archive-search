use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::ops::{ConflictAction, JournalEntry, MergePlan, MoveRecord, PlannedOp};

pub fn plan_merge(
    dir_a: &Path,
    dir_b: &Path,
    target_dir: PathBuf,
) -> Result<MergePlan, AppError> {
    let mut ops = Vec::new();
    let mut occupied: HashMap<String, PathBuf> = HashMap::new();
    let mut move_count = 0;
    let mut duplicate_count = 0;
    let mut rename_count = 0;

    for src_dir in [dir_a, dir_b] {
        for file_path in list_files(src_dir)? {
            let Some(name_os) = file_path.file_name() else {
                continue;
            };
            let name = name_os.to_string_lossy().into_owned();
            let key = name.to_ascii_lowercase();

            if let Some(existing) = occupied.get(&key) {
                if files_identical(&file_path, existing)? {
                    duplicate_count += 1;
                    ops.push(PlannedOp {
                        src: file_path,
                        dst: target_dir.join(&name),
                        action: ConflictAction::SkipDuplicate,
                    });
                } else {
                    let unique = next_available_name(&name, &occupied);
                    occupied.insert(unique.to_ascii_lowercase(), file_path.clone());
                    rename_count += 1;
                    move_count += 1;
                    ops.push(PlannedOp {
                        src: file_path,
                        dst: target_dir.join(unique),
                        action: ConflictAction::RenameSuffix,
                    });
                }
            } else {
                occupied.insert(key, file_path.clone());
                move_count += 1;
                ops.push(PlannedOp {
                    src: file_path,
                    dst: target_dir.join(&name),
                    action: ConflictAction::MoveDirect,
                });
            }
        }
    }

    Ok(MergePlan {
        target_dir,
        ops,
        move_count,
        duplicate_count,
        rename_count,
    })
}

pub fn execute_merge(plan: &MergePlan) -> Result<JournalEntry, AppError> {
    fs::create_dir_all(&plan.target_dir)?;
    let mut moves = Vec::with_capacity(plan.move_count);
    for op in &plan.ops {
        if op.action == ConflictAction::SkipDuplicate {
            continue;
        }
        move_file_safe(&op.src, &op.dst)?;
        moves.push(MoveRecord {
            from: op.src.clone(),
            to: op.dst.clone(),
        });
    }
    Ok(JournalEntry {
        created_dir: plan.target_dir.clone(),
        moves,
    })
}

pub fn move_file_safe(src: &Path, dst: &Path) -> Result<(), AppError> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    if fs::rename(src, dst).is_ok() {
        return Ok(());
    }
    fs::copy(src, dst)?;
    fs::remove_file(src)?;
    Ok(())
}

fn list_files(dir: &Path) -> Result<Vec<PathBuf>, AppError> {
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

fn files_identical(a: &Path, b: &Path) -> Result<bool, AppError> {
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

fn next_available_name(name: &str, occupied: &HashMap<String, PathBuf>) -> String {
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
