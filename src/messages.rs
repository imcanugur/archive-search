use std::path::PathBuf;
use std::sync::Arc;

use crate::index::store::ArchiveStore;
use crate::index::VisibleDir;

#[derive(Debug)]
pub enum WorkerCmd {
    ScanDir(PathBuf),
    RescanDir(PathBuf),
    ApplyBackfill(Arc<ArchiveStore>),
    Query {
        gen_id: u64,
        raw: String,
        selected_dir: Option<u32>,
    },
    Shutdown,
}

#[derive(Debug, Clone)]
pub struct SearchSnapshot {
    pub gen_id: u64,
    pub matched_files: Arc<[u32]>,
    pub visible_dirs: Arc<[VisibleDir]>,
    pub elapsed_us: u64,
}

impl Default for SearchSnapshot {
    fn default() -> Self {
        Self {
            gen_id: 0,
            matched_files: Arc::from([]),
            visible_dirs: Arc::from([]),
            elapsed_us: 0,
        }
    }
}

#[derive(Debug)]
pub enum UiEvent {
    ScanStarted,
    ScanProgress {
        dirs_found: u32,
        files_found: u32,
    },
    ScanComplete {
        store: Arc<ArchiveStore>,
        elapsed_ms: u64,
    },
    SizeProgress {
        done: u32,
        total: u32,
    },
    ScanError(String),
    SearchResult(SearchSnapshot),
}
