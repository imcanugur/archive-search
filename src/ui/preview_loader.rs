use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{Context, Key};
use image::imageops::FilterType;

use crate::index::meta::{read_media_meta, MediaMeta};
use crate::index::MediaKind;
use crate::ui::video_thumb::decode_video_rgba;

const PREVIEW_MAX_DIM: u32 = 1280;
const MAX_TEXT_BYTES: usize = 16 * 1024;

pub enum PreviewPayload {
    Image {
        width: usize,
        height: usize,
        rgba: Vec<u8>,
    },
    Text(String),
    Unsupported,
}

pub struct PreviewResult {
    pub file_id: u32,
    pub payload: PreviewPayload,
    pub meta: MediaMeta,
}

pub fn spawn_preview_loader(
    ctx: Context,
) -> (Sender<(u32, PathBuf, MediaKind)>, Receiver<PreviewResult>) {
    let (req_tx, req_rx) = unbounded::<(u32, PathBuf, MediaKind)>();
    let (resp_tx, resp_rx) = unbounded::<PreviewResult>();
    let _ = thread::Builder::new()
        .name("preview-loader".to_owned())
        .spawn(move || {
            while let Ok(mut latest) = req_rx.recv() {
                while let Ok(next) = req_rx.try_recv() {
                    latest = next;
                }
                let payload = load_preview_payload(&latest.1, latest.2);
                let meta = read_media_meta(&latest.1, latest.2);
                let _ = resp_tx.send(PreviewResult {
                    file_id: latest.0,
                    payload,
                    meta,
                });
                ctx.request_repaint();
            }
        });
    (req_tx, resp_rx)
}

fn load_preview_payload(path: &Path, kind: MediaKind) -> PreviewPayload {
    match kind {
        MediaKind::Image => load_image_preview(path),
        MediaKind::Video => decode_video_rgba(path, PREVIEW_MAX_DIM).map_or(
            PreviewPayload::Unsupported,
            |(width, height, rgba)| PreviewPayload::Image {
                width,
                height,
                rgba,
            },
        ),
        MediaKind::Other => load_text_preview(path),
    }
}

fn load_image_preview(path: &Path) -> PreviewPayload {
    let Ok(bytes) = fs::read(path) else {
        return PreviewPayload::Unsupported;
    };
    let Ok(img) = image::load_from_memory(&bytes) else {
        return PreviewPayload::Unsupported;
    };
    let resized = img
        .resize(PREVIEW_MAX_DIM, PREVIEW_MAX_DIM, FilterType::Triangle)
        .to_rgba8();
    let (w, h) = resized.dimensions();
    PreviewPayload::Image {
        width: w as usize,
        height: h as usize,
        rgba: resized.into_raw(),
    }
}

fn load_text_preview(path: &Path) -> PreviewPayload {
    let is_text = path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "txt" | "md" | "json" | "log" | "csv" | "toml"
        )
    });
    if !is_text {
        return PreviewPayload::Unsupported;
    }
    let Ok(mut f) = File::open(path) else {
        return PreviewPayload::Unsupported;
    };
    let mut buf = vec![0u8; MAX_TEXT_BYTES];
    match f.read(&mut buf) {
        Ok(n) => {
            buf.truncate(n);
            PreviewPayload::Text(String::from_utf8_lossy(&buf).into_owned())
        }
        Err(_) => PreviewPayload::Unsupported,
    }
}

pub fn handle_nav_keys(
    ctx: &Context,
    matched: &[u32],
    selected: &mut Option<u32>,
    open: &mut bool,
) {
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        *open = false;
        return;
    }
    let delta: isize = if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
        1
    } else if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
        -1
    } else {
        0
    };
    if delta == 0 || matched.is_empty() {
        return;
    }
    if let Some(pos) = (*selected).and_then(|c| matched.iter().position(|&id| id == c)) {
        let next = (pos as isize + delta).clamp(0, matched.len().saturating_sub(1) as isize);
        if let Some(&id) = matched.get(next as usize) {
            *selected = Some(id);
        }
    }
}
