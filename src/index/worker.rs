use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};

use crate::index::query::parse_query;
use crate::index::scanner::scan_directory;
use crate::index::search::execute_search;
use crate::index::store::ArchiveStore;
use crate::messages::{UiEvent, WorkerCmd};

pub fn spawn_worker(
    cmd_rx: Receiver<WorkerCmd>,
    ev_tx: Sender<UiEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) {
    let _ = thread::Builder::new()
        .name("archive-index-worker".to_owned())
        .spawn(move || worker_loop(cmd_rx, ev_tx, ctx, cancel));
}

fn worker_loop(
    cmd_rx: Receiver<WorkerCmd>,
    ev_tx: Sender<UiEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) {
    let mut store: Option<Arc<ArchiveStore>> = None;

    while let Ok(cmd) = cmd_rx.recv() {
        let latest = coalesce_latest(cmd, &cmd_rx);
        match latest {
            WorkerCmd::Shutdown => break,
            WorkerCmd::ScanDir(root) => {
                handle_scan(root, &mut store, &ev_tx, &ctx, &cancel);
            }
            WorkerCmd::Query {
                gen_id,
                raw,
                selected_dir,
            } => {
                if let Some(ref st) = store {
                    let parsed = parse_query(&raw, selected_dir);
                    let snap = execute_search(st, &parsed, gen_id);
                    let _ = ev_tx.send(UiEvent::SearchResult(snap));
                    ctx.request_repaint();
                }
            }
        }
    }
}

fn coalesce_latest(initial: WorkerCmd, cmd_rx: &Receiver<WorkerCmd>) -> WorkerCmd {
    let mut current = initial;
    while let Ok(next) = cmd_rx.try_recv() {
        match next {
            WorkerCmd::Shutdown => return WorkerCmd::Shutdown,
            WorkerCmd::ScanDir(path) => current = WorkerCmd::ScanDir(path),
            WorkerCmd::Query { .. } => {
                if matches!(current, WorkerCmd::Query { .. }) {
                    current = next;
                }
            }
        }
    }
    current
}

fn handle_scan(
    root: std::path::PathBuf,
    store: &mut Option<Arc<ArchiveStore>>,
    ev_tx: &Sender<UiEvent>,
    ctx: &egui::Context,
    cancel: &AtomicBool,
) {
    cancel.store(false, Ordering::Relaxed);
    let _ = ev_tx.send(UiEvent::ScanStarted);
    ctx.request_repaint();

    let start = Instant::now();
    match scan_directory(&root, ev_tx, ctx, cancel) {
        Ok(built) => {
            let arc = Arc::new(built);
            let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
            let parsed = parse_query("", None);
            let snap = execute_search(&arc, &parsed, 0);
            *store = Some(Arc::clone(&arc));
            let _ = ev_tx.send(UiEvent::ScanComplete {
                store: arc,
                elapsed_ms,
            });
            let _ = ev_tx.send(UiEvent::SearchResult(snap));
        }
        Err(err) => {
            let _ = ev_tx.send(UiEvent::ScanError(err.to_string()));
        }
    }
    ctx.request_repaint();
}
