use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use egui::{Context, Frame, RichText, Rounding, ScrollArea, Stroke, Ui};

use crate::gpu::vram::VectorIndex;
use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::inspector_loader::{spawn_inspector_loader, InspectorData};
use crate::ui::inspector_preview::render_preview_box;
use crate::ui::inspector_view::{render_inspector_actions, render_meta_and_tags};
use crate::ui::theme::{ON_SURFACE, OUTLINE_VARIANT, SURFACE_LOWEST};
use crate::ui::thumbs::ThumbCache;

#[derive(Default)]
pub struct InspectorAction {
    pub find_similar: Option<u32>,
    pub open_quick_look: bool,
    pub filter_tag: Option<String>,
    pub close_panel: bool,
}

pub struct InspectorPanel {
    requested_id: Option<u32>,
    data: Option<InspectorData>,
    copied_hash_id: Option<u32>,
    req_tx: Sender<(u32, PathBuf, MediaKind)>,
    resp_rx: Receiver<InspectorData>,
}

impl InspectorPanel {
    pub fn new(ctx: Context) -> Self {
        let (req_tx, resp_rx) = spawn_inspector_loader(ctx);
        Self {
            requested_id: None,
            data: None,
            copied_hash_id: None,
            req_tx,
            resp_rx,
        }
    }

    pub fn poll(&mut self) {
        while let Ok(d) = self.resp_rx.try_recv() {
            self.data = Some(d);
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        tr: &Strings,
        store: &ArchiveStore,
        fid: u32,
        vector_index: Option<&VectorIndex>,
        thumbs: &mut ThumbCache,
    ) -> InspectorAction {
        let mut act = InspectorAction::default();
        self.ensure_loaded(store, fid);
        let path = store.file_full_path(fid);
        let meta = self.data.as_ref().filter(|d| d.file_id == fid);
        let hash = meta.map_or("", |d| d.blake3_hex.as_str());
        let vec_indexed = vector_index.is_some_and(|v| v.vector_for_file(fid).is_some());
        let mut copied_id = self.copied_hash_id;

        Frame::none()
            .fill(SURFACE_LOWEST)
            .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
            .rounding(Rounding::same(8.0))
            .inner_margin(10.0)
            .show(ui, |ui| {
                render_header(ui, tr, &mut act);
                ui.separator();
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        render_preview_box(ui, store, fid, thumbs, &mut act);
                        ui.add_space(8.0);
                        render_meta_and_tags(ui, store, fid, meta, vec_indexed, &mut act);
                        ui.add_space(8.0);
                        render_inspector_actions(ui, tr, fid, &path, hash, &mut copied_id);
                    });
            });
        self.copied_hash_id = copied_id;
        act
    }

    fn ensure_loaded(&mut self, store: &ArchiveStore, fid: u32) {
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
}

fn render_header(ui: &mut Ui, tr: &Strings, act: &mut InspectorAction) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(tr.inspector_title).strong().color(ON_SURFACE));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("x").clicked() {
                act.close_panel = true;
            }
            if ui.small_button("Expand").clicked() {
                act.open_quick_look = true;
            }
        });
    });
}
