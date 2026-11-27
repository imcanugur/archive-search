use std::ffi::OsStr;
use std::time::UNIX_EPOCH;

use crossbeam_channel::Sender;

use crate::index::store::ArchiveStore;
use crate::messages::UiEvent;

pub type EntryStats = Option<(u64, i64)>;
pub type WalkEntry = jwalk::DirEntry<((), EntryStats)>;

pub fn should_skip_dir(name: &OsStr) -> bool {
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

pub fn read_entry_stats(entry: &WalkEntry) -> EntryStats {
    let meta = entry.metadata().ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .unwrap_or(0);
    Some((meta.len(), mtime))
}

pub fn extract_ext_lower(name: &str, out: &mut String) {
    out.clear();
    if let Some((_, ext)) = name.rsplit_once('.') {
        if !ext.is_empty() && ext.len() <= 16 {
            for b in ext.bytes() {
                out.push(b.to_ascii_lowercase() as char);
            }
        }
    }
}

pub fn emit_progress(store: &ArchiveStore, tx: &Sender<UiEvent>, ctx: &egui::Context) {
    let _ = tx.send(UiEvent::ScanProgress {
        dirs_found: store.dir_count() as u32,
        files_found: store.file_count() as u32,
    });
    ctx.request_repaint();
}
