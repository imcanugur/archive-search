use crate::gpu::GpuEvent;
use crate::messages::{UiEvent, WorkerCmd};

use super::ArchiveApp;

impl ArchiveApp {
    pub(super) fn drain_events(&mut self) {
        while let Ok(res) = self.telemetry_rx.try_recv() {
            self.resources = res;
        }
        self.drain_gpu_events();
        self.drain_scan_events();
    }

    fn drain_gpu_events(&mut self) {
        while let Ok(gev) = self.gpu_rx.try_recv() {
            match gev {
                GpuEvent::Progress { done, total } => {
                    self.visual_progress = Some((done, total));
                }
                GpuEvent::Complete(idx) => {
                    self.vector_index = Some(idx);
                    self.visual_progress = None;
                    if let Some(fid) = self.pending_similar.take() {
                        self.run_find_similar(fid);
                    }
                }
            }
        }
    }

    fn drain_scan_events(&mut self) {
        while let Ok(ev) = self.ev_rx.try_recv() {
            match ev {
                UiEvent::ScanStarted => self.reset_for_scan(),
                UiEvent::ScanProgress {
                    dirs_found,
                    files_found,
                } => {
                    self.scan_dirs = dirs_found;
                    self.scan_files = files_found;
                }
                UiEvent::ScanComplete { store, elapsed_ms } => {
                    self.store = Some(store);
                    self.scanning = false;
                    self.last_scan_ms = Some(elapsed_ms);
                }
                UiEvent::ScanError(msg) => {
                    self.scanning = false;
                    self.error_msg = Some(msg);
                }
                UiEvent::SearchResult(snap) => {
                    if snap.gen_id >= self.snapshot.gen_id || snap.gen_id == 0 {
                        self.snapshot = snap;
                    }
                }
            }
        }
    }

    fn reset_for_scan(&mut self) {
        self.scanning = true;
        self.scan_dirs = 0;
        self.scan_files = 0;
        self.error_msg = None;
        self.selected_dir = None;
        self.selected_file = None;
        self.vector_index = None;
        self.visual_override = None;
        self.thumbs.clear();
    }

    pub(super) fn trigger_undo(&mut self) {
        match self.journal.undo_last() {
            Ok(Some(_)) => self.rescan_current(),
            Ok(None) => {}
            Err(err) => self.error_msg = Some(err.to_string()),
        }
    }

    pub(super) fn rescan_current(&self) {
        if let Some(ref st) = self.store {
            let _ = self
                .cmd_tx
                .send(WorkerCmd::RescanDir(st.root().to_path_buf()));
        }
    }

    pub(super) fn dispatch_query(&mut self) {
        self.visual_override = None;
        self.gen_id = self.gen_id.saturating_add(1);
        let _ = self.cmd_tx.send(WorkerCmd::Query {
            gen_id: self.gen_id,
            raw: self.query.clone(),
            selected_dir: self.selected_dir,
        });
    }
}
