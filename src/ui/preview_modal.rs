use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::thread;

use crossbeam_channel::{unbounded, Receiver, Sender};
use egui::{
    ColorImage, Context, Key, RichText, ScrollArea, TextureHandle, TextureOptions, Window,
};

use crate::i18n::Strings;
use crate::index::meta::{read_media_meta, MediaMeta};
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::theme::TEXT_MUTED;

const PREVIEW_MAX_DIM: u32 = 1280;
const MAX_TEXT_BYTES: usize = 16 * 1024;

enum PreviewPayload {
    Image {
        width: usize,
        height: usize,
        rgba: Vec<u8>,
    },
    Text(String),
    Unsupported,
}

struct PreviewResult {
    file_id: u32,
    payload: PreviewPayload,
    meta: MediaMeta,
}

pub struct PreviewModal {
    open: bool,
    loaded_id: Option<u32>,
    requested_id: Option<u32>,
    texture: Option<TextureHandle>,
    text_body: Option<String>,
    meta_summary: String,
    unsupported: bool,
    req_tx: Sender<(u32, PathBuf, MediaKind)>,
    resp_rx: Receiver<PreviewResult>,
}

impl PreviewModal {
    pub fn new(ctx: Context) -> Self {
        let (req_tx, req_rx) = unbounded::<(u32, PathBuf, MediaKind)>();
        let (resp_tx, resp_rx) = unbounded::<PreviewResult>();
        let _ = thread::Builder::new()
            .name("preview-loader".to_owned())
            .spawn(move || {
                while let Ok((fid, path, kind)) = req_rx.recv() {
                    let mut latest = (fid, path, kind);
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
        Self {
            open: false,
            loaded_id: None,
            requested_id: None,
            texture: None,
            text_body: None,
            meta_summary: String::new(),
            unsupported: false,
            req_tx,
            resp_rx,
        }
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
    }

    pub fn show(
        &mut self,
        ctx: &Context,
        tr: &Strings,
        store: &ArchiveStore,
        matched: &[u32],
        selected_file: &mut Option<u32>,
    ) {
        self.poll_loaded(ctx);
        if !self.open {
            return;
        }
        handle_nav_keys(ctx, matched, selected_file, &mut self.open);
        let Some(fid) = *selected_file else {
            self.open = false;
            return;
        };
        if self.requested_id != Some(fid) {
            self.requested_id = Some(fid);
            let kind = store
                .file_kind
                .get(fid as usize)
                .copied()
                .unwrap_or(MediaKind::Other);
            let _ = self.req_tx.send((fid, store.file_full_path(fid), kind));
        }

        let mut keep = self.open;
        let mut close_clicked = false;
        Window::new(format!("{} — {}", tr.preview_title, store.file_name(fid)))
            .open(&mut keep)
            .collapsible(false)
            .resizable(true)
            .default_size([920.0, 640.0])
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(tr.preview_close).clicked() {
                            close_clicked = true;
                        }
                        if !self.meta_summary.is_empty() {
                            ui.separator();
                            ui.label(
                                RichText::new(&self.meta_summary).small().color(TEXT_MUTED),
                            );
                        }
                    });
                    ui.separator();
                    self.render_content(ui, tr, fid);
                });
            });
        self.open = keep && !close_clicked;
    }

    fn poll_loaded(&mut self, ctx: &Context) {
        while let Ok(res) = self.resp_rx.try_recv() {
            self.loaded_id = Some(res.file_id);
            self.texture = None;
            self.text_body = None;
            self.meta_summary = res.meta.format_summary();
            self.unsupported = false;
            match res.payload {
                PreviewPayload::Image {
                    width,
                    height,
                    rgba,
                } => {
                    let img = ColorImage::from_rgba_unmultiplied([width, height], &rgba);
                    self.texture =
                        Some(ctx.load_texture("preview_tex", img, TextureOptions::LINEAR));
                }
                PreviewPayload::Text(txt) => self.text_body = Some(txt),
                PreviewPayload::Unsupported => self.unsupported = true,
            }
        }
    }

    fn render_content(&self, ui: &mut egui::Ui, tr: &Strings, fid: u32) {
        if self.loaded_id != Some(fid) {
            ui.centered_and_justified(|ui| {
                ui.spinner();
                ui.label(tr.preview_loading);
            });
            return;
        }
        if let Some(ref tex) = self.texture {
            let avail = ui.available_size();
            let sz = tex.size_vec2();
            let scale = (avail.x / sz.x.max(1.0))
                .min(avail.y / sz.y.max(1.0))
                .min(1.0);
            ui.centered_and_justified(|ui| {
                ui.image((tex.id(), sz * scale));
            });
        } else if let Some(ref text) = self.text_body {
            ScrollArea::vertical().show(ui, |ui| {
                ui.label(RichText::new(text).monospace());
            });
        } else if self.unsupported {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(tr.preview_unsupported).color(TEXT_MUTED));
            });
        }
    }
}

fn handle_nav_keys(
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
    if let Some(cur) = *selected {
        if let Some(pos) = matched.iter().position(|&id| id == cur) {
            let next =
                (pos as isize + delta).clamp(0, matched.len().saturating_sub(1) as isize);
            if let Some(&id) = matched.get(next as usize) {
                *selected = Some(id);
            }
        }
    }
}

fn load_preview_payload(path: &PathBuf, kind: MediaKind) -> PreviewPayload {
    if kind == MediaKind::Image {
        if let Ok(img) = image::open(path) {
            let resized = img.thumbnail(PREVIEW_MAX_DIM, PREVIEW_MAX_DIM).to_rgba8();
            let (w, h) = resized.dimensions();
            return PreviewPayload::Image {
                width: w as usize,
                height: h as usize,
                rgba: resized.into_raw(),
            };
        }
    }
    let is_text = path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        matches!(
            e.to_ascii_lowercase().as_str(),
            "txt" | "md" | "json" | "log" | "csv" | "toml"
        )
    });
    if is_text {
        if let Ok(mut f) = File::open(path) {
            let mut buf = vec![0u8; MAX_TEXT_BYTES];
            if let Ok(n) = f.read(&mut buf) {
                buf.truncate(n);
                return PreviewPayload::Text(String::from_utf8_lossy(&buf).into_owned());
            }
        }
    }
    PreviewPayload::Unsupported
}
