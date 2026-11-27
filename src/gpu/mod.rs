pub mod similarity;
pub mod vram;

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use image::imageops::FilterType;

use crate::gpu::vram::{VectorIndex, EMBED_DIM};
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;

pub enum GpuCmd {
    IndexArchive(Arc<ArchiveStore>),
    Shutdown,
}

pub enum GpuEvent {
    Progress { done: u32, total: u32 },
    Complete(Arc<VectorIndex>),
}

pub fn spawn_gpu_worker(
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) -> (Sender<GpuCmd>, Receiver<GpuEvent>) {
    let (cmd_tx, cmd_rx) = unbounded::<GpuCmd>();
    let (ev_tx, ev_rx) = unbounded::<GpuEvent>();
    let _ = thread::Builder::new()
        .name("visual-index-worker".to_owned())
        .spawn(move || gpu_loop(cmd_rx, ev_tx, ctx, cancel));
    (cmd_tx, ev_rx)
}

fn gpu_loop(
    cmd_rx: Receiver<GpuCmd>,
    ev_tx: Sender<GpuEvent>,
    ctx: egui::Context,
    cancel: Arc<AtomicBool>,
) {
    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            GpuCmd::Shutdown => break,
            GpuCmd::IndexArchive(store) => {
                cancel.store(false, Ordering::Relaxed);
                let idx = build_visual_index(&store, &ev_tx, &ctx, &cancel);
                let _ = ev_tx.send(GpuEvent::Complete(Arc::new(idx)));
                ctx.request_repaint();
            }
        }
    }
}

fn build_visual_index(
    store: &ArchiveStore,
    ev_tx: &Sender<GpuEvent>,
    ctx: &egui::Context,
    cancel: &AtomicBool,
) -> VectorIndex {
    let image_ids: Vec<u32> = store
        .file_kind
        .iter()
        .enumerate()
        .filter_map(|(i, &k)| (k == MediaKind::Image).then_some(i as u32))
        .collect();
    let total = image_ids.len() as u32;
    let mut index = VectorIndex::new(EMBED_DIM);

    for (pos, fid) in image_ids.into_iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let path = store.file_full_path(fid);
        if let Some(vec) = extract_image_descriptor(&path) {
            index.push(fid, &vec);
        }
        if (pos + 1).is_multiple_of(32) || (pos + 1) as u32 == total {
            let _ = ev_tx.send(GpuEvent::Progress {
                done: (pos + 1) as u32,
                total,
            });
            ctx.request_repaint();
        }
    }
    index
}

fn extract_image_descriptor(path: &Path) -> Option<[f32; EMBED_DIM]> {
    let bytes = fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let small = img.resize_exact(16, 16, FilterType::Triangle).to_rgb8();
    let mut desc = [0.0_f32; EMBED_DIM];

    for (y, row) in small.rows().enumerate() {
        let by = y / 4;
        for (x, px) in row.enumerate() {
            let bx = x / 4;
            let cell = by * 4 + bx;
            let base = cell * 4;
            let [r, g, b] = px.0;
            let rf = f32::from(r) / 255.0;
            let gf = f32::from(g) / 255.0;
            let bf = f32::from(b) / 255.0;
            let luma = 0.299 * rf + 0.587 * gf + 0.114 * bf;
            if let Some(slot) = desc.get_mut(base..base + 4) {
                slot[0] += rf;
                slot[1] += gf;
                slot[2] += bf;
                slot[3] += luma;
            }
        }
    }
    Some(desc)
}
