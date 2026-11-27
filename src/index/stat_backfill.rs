use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::UNIX_EPOCH;

use crossbeam_channel::Sender;
use rayon::prelude::*;

use crate::index::arena::ByteArena;
use crate::index::persistence::save_store;
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

fn stat_file_meta(path: &Path) -> (u64, i64) {
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
                    let (cs, cm) = (
                        sz.get(i).copied().unwrap_or(0),
                        mt.get(i).copied().unwrap_or(0),
                    );
                    stat_one(store, &dir_paths, i, cs, cm)
                })
                .collect()
        });
        apply_chunk(&mut sz, &mut mt, &mut yr, start, &batch);
        let snap = (end == count || (chunk_idx + 1).is_multiple_of(PUBLISH_EVERY))
            .then(|| clone_enriched(store, &sz, &mt, &yr));
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
        if let Some(y) = yr.get_mut(i) {
            if *y == 0 && m > 1 {
                *y = epoch_secs_to_year(m);
            }
        }
    }
}

fn clone_enriched(store: &ArchiveStore, sz: &[u64], mt: &[i64], yr: &[u16]) -> ArchiveStore {
    ArchiveStore {
        root: store.root.clone(),
        raw_arena: ByteArena::from_bytes(store.raw_arena.as_bytes().to_vec()),
        fold_arena: ByteArena::from_bytes(store.fold_arena.as_bytes().to_vec()),
        ext_table: store.ext_table.clone(),
        ext_map: store.ext_map.clone(),
        cam_table: store.cam_table.clone(),
        dir_parent: store.dir_parent.clone(),
        dir_raw_name: store.dir_raw_name.clone(),
        dir_fold_name: store.dir_fold_name.clone(),
        dir_depth: store.dir_depth.clone(),
        file_dir: store.file_dir.clone(),
        file_raw_name: store.file_raw_name.clone(),
        file_fold_name: store.file_fold_name.clone(),
        file_ext: store.file_ext.clone(),
        file_size: sz.to_vec(),
        file_mtime: mt.to_vec(),
        file_year: yr.to_vec(),
        file_cam: store.file_cam.clone(),
        file_kind: store.file_kind.clone(),
    }
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
