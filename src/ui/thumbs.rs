use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::{unbounded, Receiver};
use egui::{ColorImage, Context, TextureHandle, TextureOptions};

use crate::index::MediaKind;
use crate::ui::thumb_queue::{spawn_lane, LifoQueue, ThumbReq, ThumbResp};

const MAX_CACHED_TEXTURES: usize = 256;
const MAX_UPLOADS_PER_FRAME: usize = 16;

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
            let req = ThumbReq {
                file_id,
                path,
                kind,
            };
            if let Some(evicted) = q.push(req) {
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
