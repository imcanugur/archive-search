use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::Sender;
use rayon::prelude::*;

use crate::index::persistence::save_store;
use crate::index::scan_helpers::{epoch_secs_to_year, stat_file_meta};
use crate::index::store::ArchiveStore;
use crate::messages::{UiEvent, WorkerCmd};

const STAT_CHUNK: usize = 4096;
const PUBLISH_EVERY: usize = 8;
const IO_THREADS: usize = 32;

pub fn spawn_stat_backfill(
    store: Arc<ArchiveStore>,
    cmd_tx: Sender<WorkerCmd>,
    ev_tx: Sender<UiEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) {
    let _ = thread::Builder::new()
        .name("archive-stat-backfill".to_owned())
        .spawn(move || {
            backfill_store_stats(&store, &cancel, |done, total, partial| {
                let _ = ev_tx.send(UiEvent::SizeProgress { done, total });
                if let Some(enriched) = partial {
                    let _ = save_store(&enriched);
                    let _ = cmd_tx.send(WorkerCmd::ApplyBackfill(Arc::new(enriched)));
                }
                ctx.request_repaint();
            });
        });
}

fn backfill_store_stats<F>(store: &ArchiveStore, cancel: &AtomicBool, mut on_batch: F)
where
    F: FnMut(u32, u32, Option<ArchiveStore>),
{
    let count = store.file_count();
    let Ok(pool) = rayon::ThreadPoolBuilder::new()
        .num_threads(IO_THREADS)
        .build()
    else {
        return;
    };
    let dir_paths: Vec<PathBuf> = (0..store.dir_count())
        .map(|d| store.dir_full_path(d as u32))
        .collect();
    let (mut sz, mut mt, mut yr) = (
        store.file_size.clone(),
        store.file_mtime.clone(),
        store.file_year.clone(),
    );
    for chunk_idx in 0..count.div_ceil(STAT_CHUNK) {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        let start = chunk_idx * STAT_CHUNK;
        let end = (start + STAT_CHUNK).min(count);
        let batch: Vec<(u64, i64)> = pool.install(|| {
            (start..end)
                .into_par_iter()
                .map(|i| {
                    let cs = sz.get(i).copied().unwrap_or(0);
                    let cm = mt.get(i).copied().unwrap_or(0);
                    stat_one(store, &dir_paths, i, cs, cm)
                })
                .collect()
        });
        apply_chunk(&mut sz, &mut mt, &mut yr, start, &batch);
        let snap = (end == count || (chunk_idx + 1).is_multiple_of(PUBLISH_EVERY))
            .then(|| store.with_stats(sz.clone(), mt.clone(), yr.clone()));
        on_batch(end as u32, count as u32, snap);
    }
}

fn stat_one(store: &ArchiveStore, dirs: &[PathBuf], i: usize, s: u64, m: i64) -> (u64, i64) {
    if m != 0 {
        return (s, m);
    }
    let d = store.file_dir.get(i).copied().unwrap_or(0) as usize;
    dirs.get(d).map_or((0, 1), |dir| {
        stat_file_meta(&dir.join(store.file_name(i as u32)))
    })
}

fn apply_chunk(
    sz: &mut [u64],
    mt: &mut [i64],
    yr: &mut [u16],
    start: usize,
    batch: &[(u64, i64)],
) {
    for (off, &(s, m)) in batch.iter().enumerate() {
        let i = start + off;
        if let Some(dst) = sz.get_mut(i) {
            *dst = s;
        }
        if let Some(dst) = mt.get_mut(i) {
            *dst = m;
        }
        if let Some(y) = yr.get_mut(i).filter(|y| **y == 0 && m > 1) {
            *y = epoch_secs_to_year(m);
        }
    }
}
