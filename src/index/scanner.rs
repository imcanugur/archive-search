use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

use crossbeam_channel::Sender;
use jwalk::WalkDirGeneric;

use crate::error::AppError;
use crate::index::store::{ArchiveStore, FileRecord};
use crate::messages::UiEvent;

const PROGRESS_BATCH: u32 = 2048;

type EntryStats = Option<(u64, i64)>;
type WalkEntry = jwalk::DirEntry<((), EntryStats)>;

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
    let read_meta = !root.starts_with("/mnt/");
    let walker = WalkDirGeneric::<((), EntryStats)>::new(root)
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
        });

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

fn should_skip_dir(name: &OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(
            "$RECYCLE.BIN"
                | "System Volume Information"
                | "Recovery"
                | "Config.Msi"
                | "Windows"
                | "Program Files"
                | "Program Files (x86)"
                | "ProgramData"
                | ".git"
                | "node_modules"
                | "target"
                | ".Trash-1000"
                | "proc"
                | "sys"
                | "dev"
        )
    )
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

fn read_entry_stats(entry: &WalkEntry) -> EntryStats {
    let meta = entry.metadata().ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    Some((meta.len(), mtime))
}

fn extract_ext_lower(name: &str, out: &mut String) {
    out.clear();
    if let Some((_, ext)) = name.rsplit_once('.') {
        if !ext.is_empty() && ext.len() <= 16 {
            for b in ext.bytes() {
                out.push(b.to_ascii_lowercase() as char);
            }
        }
    }
}

fn emit_progress(store: &ArchiveStore, tx: &Sender<UiEvent>, ctx: &egui::Context) {
    let _ = tx.send(UiEvent::ScanProgress {
        dirs_found: store.dir_count() as u32,
        files_found: store.file_count() as u32,
    });
    ctx.request_repaint();
}
