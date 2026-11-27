use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use egui::{ColorImage, Context, RichText, ScrollArea, TextureHandle, TextureOptions, Window};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::preview_loader::{
    handle_nav_keys, spawn_preview_loader, PreviewPayload, PreviewResult,
};
use crate::ui::theme::TEXT_MUTED;

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
        let (req_tx, resp_rx) = spawn_preview_loader(ctx);
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
        self.ensure_requested(store, fid);

        let mut keep = self.open;
        let mut close_clicked = false;
        let title = format!("{} — {}", tr.preview_title, store.file_name(fid));
        Window::new(title)
            .open(&mut keep)
            .collapsible(false)
            .resizable(true)
            .default_size([920.0, 640.0])
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        close_clicked = ui.button(tr.preview_close).clicked();
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

    fn ensure_requested(&mut self, store: &ArchiveStore, fid: u32) {
        if self.requested_id == Some(fid) {
            return;
        }
        self.requested_id = Some(fid);
        let kind = store
            .file_kind
            .get(fid as usize)
            .copied()
            .unwrap_or(MediaKind::Other);
        let _ = self.req_tx.send((fid, store.file_full_path(fid), kind));
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
            ui.centered_and_justified(|ui| ui.image((tex.id(), sz * scale)));
        } else if let Some(ref text) = self.text_body {
            ScrollArea::vertical().show(ui, |ui| ui.label(RichText::new(text).monospace()));
        } else if self.unsupported {
            ui.centered_and_justified(|ui| {
                ui.label(RichText::new(tr.preview_unsupported).color(TEXT_MUTED));
            });
        }
    }
}
