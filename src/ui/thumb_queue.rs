use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossbeam_channel::Sender;
use egui::Context;
use image::imageops::FilterType;

use crate::index::MediaKind;
use crate::ui::video_thumb::decode_video_rgba;

pub const MAX_PENDING: usize = 32;
pub const THUMB_DIM: u32 = 160;

pub struct ThumbReq {
    pub file_id: u32,
    pub path: PathBuf,
    pub kind: MediaKind,
}

pub struct ThumbResp {
    pub file_id: u32,
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

pub struct LifoQueue {
    items: Mutex<Vec<ThumbReq>>,
    ready: Condvar,
}

impl LifoQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            items: Mutex::new(Vec::with_capacity(MAX_PENDING)),
            ready: Condvar::new(),
        })
    }

    pub fn push(&self, req: ThumbReq) -> Option<u32> {
        let Ok(mut guard) = self.items.lock() else {
            return None;
        };
        let evicted = (guard.len() >= MAX_PENDING).then(|| guard.remove(0).file_id);
        guard.push(req);
        self.ready.notify_one();
        evicted
    }

    pub fn pop(&self) -> Option<ThumbReq> {
        let mut guard = self.items.lock().ok()?;
        while guard.is_empty() {
            guard = self.ready.wait(guard).ok()?;
        }
        guard.pop()
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.items.lock() {
            guard.clear();
        }
    }
}

pub fn spawn_lane(
    q: Arc<LifoQueue>,
    tx: Sender<ThumbResp>,
    ctx: Context,
    tag: &str,
    count: usize,
) {
    for idx in 0..count {
        let q_c = Arc::clone(&q);
        let tx_c = tx.clone();
        let ctx_c = ctx.clone();
        let _ = thread::Builder::new()
            .name(format!("thumb-{tag}-{idx}"))
            .spawn(move || {
                while let Some(req) = q_c.pop() {
                    let _ = tx_c.send(decode_thumb(req));
                    ctx_c.request_repaint();
                }
            });
    }
}

fn decode_thumb(req: ThumbReq) -> ThumbResp {
    let empty = ThumbResp {
        file_id: req.file_id,
        width: 0,
        height: 0,
        rgba: Vec::new(),
    };
    let decoded = match req.kind {
        MediaKind::Image => decode_image_rgba(&req.path),
        MediaKind::Video => decode_video_rgba(&req.path, THUMB_DIM),
        MediaKind::Other => None,
    };
    let Some((width, height, rgba)) = decoded else {
        return empty;
    };
    ThumbResp {
        file_id: req.file_id,
        width,
        height,
        rgba,
    }
}

fn decode_image_rgba(path: &Path) -> Option<(usize, usize, Vec<u8>)> {
    let bytes = fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let thumb = img
        .resize(THUMB_DIM, THUMB_DIM, FilterType::Triangle)
        .to_rgba8();
    let (w, h) = thumb.dimensions();
    Some((w as usize, h as usize, thumb.into_raw()))
}
