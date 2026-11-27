use std::ffi::OsStr;
use std::path::Path;
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

pub fn stat_file_meta(path: &Path) -> (u64, i64) {
    let Ok(meta) = std::fs::metadata(path) else {
        return (0, 1);
    };
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .filter(|&s| s > 0)
        .unwrap_or(1);
    (meta.len(), mtime)
}

pub fn epoch_secs_to_year(secs: i64) -> u16 {
    if secs <= 1 {
        return 0;
    }
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    u16::try_from(yoe + era * 400 + i64::from(m <= 2)).unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::epoch_secs_to_year;

    #[test]
    fn converts_unix_timestamp_to_year() {
        assert_eq!(epoch_secs_to_year(1_704_067_200), 2024);
        assert_eq!(epoch_secs_to_year(1_735_689_600), 2025);
    }
}
