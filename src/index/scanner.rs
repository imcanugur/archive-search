use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crossbeam_channel::Sender;
use jwalk::WalkDirGeneric;

use crate::error::AppError;
use crate::index::scan_helpers::{
    emit_progress, extract_ext_lower, read_entry_stats, should_skip_dir, EntryStats, WalkEntry,
};
use crate::index::store::{ArchiveStore, FileRecord};
use crate::messages::UiEvent;

const PROGRESS_BATCH: u32 = 2048;

pub fn scan_directory(
    root: &Path,
    tx: &Sender<UiEvent>,
    ctx: &egui::Context,
    cancel: &AtomicBool,
) -> Result<ArchiveStore, AppError> {
    if !root.exists() {
        return Err(AppError::RootNotFound(root.to_path_buf()));
    }
    let _meta = std::fs::metadata(root)?;
    let mut store = ArchiveStore::new(root.to_path_buf());
    let mut fold_buf = String::with_capacity(128);
    let root_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    store.add_dir(0, root_name, 0, &mut fold_buf)?;

    let mut dir_ids: HashMap<PathBuf, u32> = HashMap::with_capacity(4096);
    dir_ids.insert(root.to_path_buf(), 0);
    walk_entries(
        root,
        &mut store,
        &mut dir_ids,
        &mut fold_buf,
        tx,
        ctx,
        cancel,
    )?;
    Ok(store)
}

fn walk_entries(
    root: &Path,
    store: &mut ArchiveStore,
    dir_ids: &mut HashMap<PathBuf, u32>,
    fold_buf: &mut String,
    tx: &Sender<UiEvent>,
    ctx: &egui::Context,
    cancel: &AtomicBool,
) -> Result<(), AppError> {
    let walker = build_walker(root);
    let mut seen: u32 = 0;
    let mut ext_buf = String::with_capacity(16);
    for entry_res in walker {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let Ok(entry) = entry_res else {
            continue;
        };
        if entry.depth() == 0 {
            continue;
        }
        ingest_entry(&entry, store, dir_ids, fold_buf, &mut ext_buf)?;
        seen = seen.saturating_add(1);
        if seen.is_multiple_of(PROGRESS_BATCH) {
            emit_progress(store, tx, ctx);
        }
    }
    emit_progress(store, tx, ctx);
    Ok(())
}

fn build_walker(root: &Path) -> WalkDirGeneric<((), EntryStats)> {
    let read_meta = !root.starts_with("/mnt/");
    WalkDirGeneric::<((), EntryStats)>::new(root)
        .skip_hidden(false)
        .sort(true)
        .process_read_dir(move |_depth, _path, _state, children| {
            for entry in children.iter_mut().flatten() {
                if entry.file_type.is_dir() && should_skip_dir(&entry.file_name) {
                    entry.read_children_path = None;
                } else if read_meta && entry.file_type.is_file() {
                    entry.client_state = read_entry_stats(entry);
                }
            }
        })
}

fn ingest_entry(
    entry: &WalkEntry,
    store: &mut ArchiveStore,
    dir_ids: &mut HashMap<PathBuf, u32>,
    fold_buf: &mut String,
    ext_buf: &mut String,
) -> Result<(), AppError> {
    let parent_id = dir_ids.get(entry.parent_path()).copied().unwrap_or(0);
    let name = entry.file_name().to_string_lossy();
    let ftype = entry.file_type();

    if ftype.is_dir() {
        if should_skip_dir(entry.file_name()) {
            return Ok(());
        }
        let depth = u16::try_from(entry.depth()).unwrap_or(u16::MAX);
        let dir_id = store.add_dir(parent_id, &name, depth, fold_buf)?;
        dir_ids.insert(entry.path(), dir_id);
        return Ok(());
    }
    if !ftype.is_file() {
        return Ok(());
    }
    let (size, mtime) = entry.client_state.unwrap_or((0, 0));
    extract_ext_lower(&name, ext_buf);
    let rec = FileRecord {
        dir_id: parent_id,
        name: &name,
        ext: ext_buf,
        size,
        mtime,
    };
    let _ = store.add_file(rec, fold_buf)?;
    Ok(())
}
