use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;
use crate::ops::hash_util::{files_identical, list_files, next_available_name};
use crate::ops::{ConflictAction, JournalEntry, MergePlan, MoveRecord, PlannedOp};

pub fn plan_merge(
    dir_a: &Path,
    dir_b: &Path,
    target_dir: PathBuf,
) -> Result<MergePlan, AppError> {
    let mut plan = MergePlan {
        target_dir,
        ops: Vec::new(),
        move_count: 0,
        duplicate_count: 0,
        rename_count: 0,
    };
    let mut occupied: HashMap<String, PathBuf> = HashMap::new();
    for src_dir in [dir_a, dir_b] {
        for file_path in list_files(src_dir)? {
            plan_single_file(file_path, &mut occupied, &mut plan)?;
        }
    }
    Ok(plan)
}

fn plan_single_file(
    file_path: PathBuf,
    occupied: &mut HashMap<String, PathBuf>,
    plan: &mut MergePlan,
) -> Result<(), AppError> {
    let Some(name_os) = file_path.file_name() else {
        return Ok(());
    };
    let name = name_os.to_string_lossy().into_owned();
    let key = name.to_ascii_lowercase();
    let Some(existing) = occupied.get(&key) else {
        occupied.insert(key, file_path.clone());
        plan.move_count += 1;
        plan.ops.push(PlannedOp {
            src: file_path,
            dst: plan.target_dir.join(&name),
            action: ConflictAction::MoveDirect,
        });
        return Ok(());
    };
    if files_identical(&file_path, existing)? {
        plan.duplicate_count += 1;
        plan.ops.push(PlannedOp {
            src: file_path,
            dst: plan.target_dir.join(&name),
            action: ConflictAction::SkipDuplicate,
        });
        return Ok(());
    }
    let unique = next_available_name(&name, occupied);
    occupied.insert(unique.to_ascii_lowercase(), file_path.clone());
    plan.rename_count += 1;
    plan.move_count += 1;
    plan.ops.push(PlannedOp {
        src: file_path,
        dst: plan.target_dir.join(unique),
        action: ConflictAction::RenameSuffix,
    });
    Ok(())
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
