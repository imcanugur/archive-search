use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender};

use crate::index::persistence::{load_store, save_store};
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
                handle_scan(root, true, &mut store, &ev_tx, &ctx, &cancel);
            }
            WorkerCmd::RescanDir(root) => {
                handle_scan(root, false, &mut store, &ev_tx, &ctx, &cancel);
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
            WorkerCmd::ScanDir(p) => current = WorkerCmd::ScanDir(p),
            WorkerCmd::RescanDir(p) => current = WorkerCmd::RescanDir(p),
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
    root: PathBuf,
    use_cache: bool,
    store: &mut Option<Arc<ArchiveStore>>,
    ev_tx: &Sender<UiEvent>,
    ctx: &egui::Context,
    cancel: &AtomicBool,
) {
    cancel.store(false, Ordering::Relaxed);
    let _ = ev_tx.send(UiEvent::ScanStarted);
    ctx.request_repaint();

    let start = Instant::now();
    if let Some(cached) = use_cache.then(|| load_store(&root)).flatten() {
        publish_store(cached, start, store, ev_tx, ctx);
        return;
    }
    match scan_directory(&root, ev_tx, ctx, cancel) {
        Ok(built) => {
            if !cancel.load(Ordering::Relaxed) {
                let _ = save_store(&built);
            }
            publish_store(built, start, store, ev_tx, ctx);
        }
        Err(err) => {
            let _ = ev_tx.send(UiEvent::ScanError(err.to_string()));
            ctx.request_repaint();
        }
    }
}

fn publish_store(
    built: ArchiveStore,
    start: Instant,
    store: &mut Option<Arc<ArchiveStore>>,
    ev_tx: &Sender<UiEvent>,
    ctx: &egui::Context,
) {
    let arc = Arc::new(built);
    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let snap = execute_search(&arc, &parse_query("", None), 0);
    *store = Some(Arc::clone(&arc));
    let _ = ev_tx.send(UiEvent::ScanComplete {
        store: arc,
        elapsed_ms,
    });
    let _ = ev_tx.send(UiEvent::SearchResult(snap));
    ctx.request_repaint();
}
