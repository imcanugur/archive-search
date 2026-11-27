use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::Context;

use crate::index::meta::{read_media_meta, MediaMeta};
use crate::index::MediaKind;

pub struct InspectorData {
    pub file_id: u32,
    pub meta: MediaMeta,
    pub blake3_hex: String,
    pub file_size: u64,
}

pub fn spawn_inspector_loader(
    ctx: Context,
) -> (Sender<(u32, PathBuf, MediaKind)>, Receiver<InspectorData>) {
    let (req_tx, req_rx) = unbounded::<(u32, PathBuf, MediaKind)>();
    let (resp_tx, resp_rx) = unbounded::<InspectorData>();
    let _ = thread::Builder::new()
        .name("inspector-loader".to_owned())
        .spawn(move || {
            while let Ok(mut latest) = req_rx.recv() {
                while let Ok(next) = req_rx.try_recv() {
                    latest = next;
                }
                let data = load_inspector_data(latest.0, &latest.1, latest.2);
                let _ = resp_tx.send(data);
                ctx.request_repaint();
            }
        });
    (req_tx, resp_rx)
}

fn load_inspector_data(file_id: u32, path: &Path, kind: MediaKind) -> InspectorData {
    let meta = read_media_meta(path, kind);
    let file_size = std::fs::metadata(path).map_or(0, |m| m.len());
    let blake3_hex = compute_quick_blake3(path);
    InspectorData {
        file_id,
        meta,
        blake3_hex,
        file_size,
    }
}

fn compute_quick_blake3(path: &Path) -> String {
    let Ok(mut f) = File::open(path) else {
        return String::new();
    };
    let mut hasher = blake3::Hasher::new();
    let mut buf = [0u8; 64 * 1024];
    let mut hashed = 0usize;
    while hashed < 2 * 1024 * 1024 {
        let Ok(n) = f.read(&mut buf) else {
            break;
        };
        if n == 0 {
            break;
        }
        if let Some(slice) = buf.get(..n) {
            hasher.update(slice);
        }
        hashed += n;
    }
    hasher.finalize().to_hex().to_string()
}
