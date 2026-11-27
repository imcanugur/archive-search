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
use crate::index::stat_backfill::spawn_stat_backfill;
use crate::index::store::ArchiveStore;
use crate::messages::{UiEvent, WorkerCmd};

struct WorkerCtx {
    cmd_tx: Sender<WorkerCmd>,
    ev_tx: Sender<UiEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
    store: Option<Arc<ArchiveStore>>,
    last_query: (u64, String, Option<u32>),
}

pub fn spawn_worker(
    cmd_tx: Sender<WorkerCmd>,
    cmd_rx: Receiver<WorkerCmd>,
    ev_tx: Sender<UiEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) {
    let w = WorkerCtx {
        cmd_tx,
        ev_tx,
        ctx,
        cancel,
        store: None,
        last_query: (0, String::new(), None),
    };
    let _ = thread::Builder::new()
        .name("archive-index-worker".to_owned())
        .spawn(move || worker_loop(w, cmd_rx));
}

fn worker_loop(mut w: WorkerCtx, cmd_rx: Receiver<WorkerCmd>) {
    while let Ok(cmd) = cmd_rx.recv() {
        match coalesce_latest(cmd, &cmd_rx) {
            WorkerCmd::Shutdown => break,
            WorkerCmd::ScanDir(root) => handle_scan(&mut w, root, true),
            WorkerCmd::RescanDir(root) => handle_scan(&mut w, root, false),
            WorkerCmd::ApplyBackfill(enriched) => apply_backfill(&mut w, enriched),
            WorkerCmd::Query {
                gen_id,
                raw,
                selected_dir,
            } => {
                w.last_query = (gen_id, raw.clone(), selected_dir);
                run_query(&w, gen_id, &raw, selected_dir);
            }
        }
    }
}

fn run_query(w: &WorkerCtx, gen_id: u64, raw: &str, selected_dir: Option<u32>) {
    if let Some(ref st) = w.store {
        let parsed = parse_query(raw, selected_dir);
        let snap = execute_search(st, &parsed, gen_id);
        let _ = w.ev_tx.send(UiEvent::SearchResult(snap));
        w.ctx.request_repaint();
    }
}

fn coalesce_latest(initial: WorkerCmd, cmd_rx: &Receiver<WorkerCmd>) -> WorkerCmd {
    let mut current = initial;
    while let Ok(next) = cmd_rx.try_recv() {
        match next {
            WorkerCmd::Shutdown => return WorkerCmd::Shutdown,
            WorkerCmd::ScanDir(p) => current = WorkerCmd::ScanDir(p),
            WorkerCmd::RescanDir(p) => current = WorkerCmd::RescanDir(p),
            WorkerCmd::ApplyBackfill(st) => {
                if !matches!(current, WorkerCmd::ScanDir(_) | WorkerCmd::RescanDir(_)) {
                    current = WorkerCmd::ApplyBackfill(st);
                }
            }
            WorkerCmd::Query { .. } if matches!(current, WorkerCmd::Query { .. }) => {
                current = next;
            }
            WorkerCmd::Query { .. } => {}
        }
    }
    current
}

fn handle_scan(w: &mut WorkerCtx, root: PathBuf, use_cache: bool) {
    w.cancel.store(false, Ordering::Relaxed);
    w.last_query = (0, String::new(), None);
    let _ = w.ev_tx.send(UiEvent::ScanStarted);
    w.ctx.request_repaint();

    let start = Instant::now();
    if let Some(cached) = use_cache.then(|| load_store(&root)).flatten() {
        publish_store(w, cached, start);
        return;
    }
    match scan_directory(&root, &w.ev_tx, &w.ctx, &w.cancel) {
        Ok(built) => {
            if !w.cancel.load(Ordering::Relaxed) {
                let _ = save_store(&built);
            }
            publish_store(w, built, start);
        }
        Err(err) => {
            let _ = w.ev_tx.send(UiEvent::ScanError(err.to_string()));
            w.ctx.request_repaint();
        }
    }
}

fn publish_store(w: &mut WorkerCtx, built: ArchiveStore, start: Instant) {
    let arc = Arc::new(built);
    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let snap = execute_search(&arc, &parse_query("", None), 0);
    w.store = Some(Arc::clone(&arc));
    if !arc.sizes_populated() {
        spawn_stat_backfill(
            Arc::clone(&arc),
            w.cmd_tx.clone(),
            w.ev_tx.clone(),
            w.ctx.clone(),
            Arc::clone(&w.cancel),
        );
    }
    let _ = w.ev_tx.send(UiEvent::ScanComplete {
        store: arc,
        elapsed_ms,
    });
    let _ = w.ev_tx.send(UiEvent::SearchResult(snap));
    w.ctx.request_repaint();
}

fn apply_backfill(w: &mut WorkerCtx, enriched: Arc<ArchiveStore>) {
    if w.store
        .as_ref()
        .is_none_or(|cur| cur.root() != enriched.root())
    {
        return;
    }
    w.store = Some(Arc::clone(&enriched));
    let _ = w.ev_tx.send(UiEvent::ScanComplete {
        store: enriched,
        elapsed_ms: 0,
    });
    let (gen_id, ref raw, sel) = w.last_query;
    run_query(w, gen_id, raw, sel);
}
