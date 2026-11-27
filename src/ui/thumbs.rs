use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use image::imageops::FilterType;

use crate::index::MediaKind;
use crate::ui::video_thumb::decode_video_rgba;

const MAX_CACHED_TEXTURES: usize = 256;
const MAX_PENDING: usize = 32;
const MAX_UPLOADS_PER_FRAME: usize = 16;
const THUMB_DIM: u32 = 160;

struct ThumbReq {
    file_id: u32,
    path: PathBuf,
    kind: MediaKind,
}

struct ThumbResp {
    file_id: u32,
    width: usize,
    height: usize,
    rgba: Vec<u8>,
}

struct LifoQueue {
    items: Mutex<Vec<ThumbReq>>,
    ready: Condvar,
}

impl LifoQueue {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            items: Mutex::new(Vec::with_capacity(MAX_PENDING)),
            ready: Condvar::new(),
        })
    }

    fn push(&self, req: ThumbReq) -> Option<u32> {
        let Ok(mut guard) = self.items.lock() else {
            return None;
        };
        let evicted = if guard.len() >= MAX_PENDING {
            Some(guard.remove(0).file_id)
        } else {
            None
        };
        guard.push(req);
        self.ready.notify_one();
        evicted
    }

    fn pop(&self) -> Option<ThumbReq> {
        let mut guard = self.items.lock().ok()?;
        while guard.is_empty() {
            guard = self.ready.wait(guard).ok()?;
        }
        guard.pop()
    }

    fn clear(&self) {
        if let Ok(mut guard) = self.items.lock() {
            guard.clear();
        }
    }
}

pub struct ThumbCache {
    img_q: Arc<LifoQueue>,
    vid_q: Arc<LifoQueue>,
    resp_rx: Receiver<ThumbResp>,
    textures: HashMap<u32, TextureHandle>,
    order: VecDeque<u32>,
    in_flight: HashSet<u32>,
    failed: HashSet<u32>,
}

impl ThumbCache {
    pub fn new(ctx: Context) -> Self {
        let img_q = LifoQueue::new();
        let vid_q = LifoQueue::new();
        let (resp_tx, resp_rx) = unbounded::<ThumbResp>();
        spawn_lane(Arc::clone(&img_q), resp_tx.clone(), ctx.clone(), "img", 2);
        spawn_lane(Arc::clone(&vid_q), resp_tx, ctx, "vid", 3);
        Self {
            img_q,
            vid_q,
            resp_rx,
            textures: HashMap::with_capacity(MAX_CACHED_TEXTURES),
            order: VecDeque::with_capacity(MAX_CACHED_TEXTURES),
            in_flight: HashSet::new(),
            failed: HashSet::new(),
        }
    }

    pub fn vram_bytes(&self) -> u64 {
        self.textures
            .values()
            .map(|t| {
                let [w, h] = t.size();
                (w as u64).saturating_mul(h as u64).saturating_mul(4)
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.img_q.clear();
        self.vid_q.clear();
        self.textures.clear();
        self.order.clear();
        self.in_flight.clear();
        self.failed.clear();
    }

    pub fn poll_uploads(&mut self, ctx: &Context) {
        for _ in 0..MAX_UPLOADS_PER_FRAME {
            let Ok(resp) = self.resp_rx.try_recv() else {
                break;
            };
            self.in_flight.remove(&resp.file_id);
            if resp.rgba.is_empty() {
                self.failed.insert(resp.file_id);
                continue;
            }
            let img =
                ColorImage::from_rgba_unmultiplied([resp.width, resp.height], &resp.rgba);
            let h = ctx.load_texture(
                format!("thumb_{}", resp.file_id),
                img,
                TextureOptions::LINEAR,
            );
            self.insert_lru(resp.file_id, h);
        }
    }

    pub fn get_or_request(
        &mut self,
        file_id: u32,
        path: PathBuf,
        kind: MediaKind,
    ) -> Option<&TextureHandle> {
        if self.textures.contains_key(&file_id) {
            return self.textures.get(&file_id);
        }
        if !self.in_flight.contains(&file_id) && !self.failed.contains(&file_id) {
            let q = if kind == MediaKind::Video {
                &self.vid_q
            } else {
                &self.img_q
            };
            if let Some(evicted) = q.push(ThumbReq {
                file_id,
                path,
                kind,
            }) {
                self.in_flight.remove(&evicted);
            }
            self.in_flight.insert(file_id);
        }
        None
    }

    fn insert_lru(&mut self, file_id: u32, handle: TextureHandle) {
        while self.order.len() >= MAX_CACHED_TEXTURES {
            if let Some(oldest) = self.order.pop_front() {
                self.textures.remove(&oldest);
            }
        }
        self.textures.insert(file_id, handle);
        self.order.push_back(file_id);
    }
}

fn spawn_lane(
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
