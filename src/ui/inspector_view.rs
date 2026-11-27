use std::path::Path;

use egui::{Button, Color32, Frame, RichText, Rounding, Ui, Vec2};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::index::MediaKind;
use crate::ui::format_bytes;
use crate::ui::inspector_loader::InspectorData;
use crate::ui::inspector_panel::InspectorAction;
use crate::ui::os_open::open_path;
use crate::ui::theme::{
    INVERSE_SURFACE, INVERSE_TEXT, ON_SURFACE, ON_SURFACE_VARIANT, PRIMARY, SECONDARY,
    SURFACE_CONTAINER, SURFACE_LOW, TERTIARY,
};
use crate::ui::thumbs::ThumbCache;

pub fn render_preview_box(
    ui: &mut Ui,
    fid: u32,
    kind: MediaKind,
    path: &Path,
    thumbs: &mut ThumbCache,
    act: &mut InspectorAction,
) {
    let tex = matches!(kind, MediaKind::Image | MediaKind::Video)
        .then(|| thumbs.get_or_request(fid, path.to_path_buf(), kind))
        .flatten();
    let box_sz = Vec2::new(ui.available_width(), 150.0);
    Frame::none()
        .fill(INVERSE_SURFACE)
        .rounding(Rounding::same(6.0))
        .show(ui, |ui| {
            ui.allocate_ui(box_sz, |ui| {
                ui.centered_and_justified(|ui| match tex {
                    Some(t) => {
                        let sz = t.size_vec2();
                        let s = (box_sz.x / sz.x.max(1.0)).min(box_sz.y / sz.y.max(1.0));
                        ui.image((t.id(), sz * s));
                    }
                    None => {
                        ui.label(RichText::new(kind.badge()).monospace().color(INVERSE_TEXT));
                    }
                });
            });
        });
    ui.add_space(4.0);
    let sim = Button::new(RichText::new("Visually Similar").color(PRIMARY));
    if ui.add_sized([ui.available_width(), 26.0], sim).clicked() {
        act.find_similar = Some(fid);
    }
}

pub fn render_meta_and_tags(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    meta: Option<&InspectorData>,
    act: &mut InspectorAction,
) {
    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let size = meta.map_or_else(
        || store.file_size.get(fid as usize).copied().unwrap_or(0),
        |d| d.file_size,
    );
    Frame::none()
        .fill(SURFACE_LOW)
        .rounding(Rounding::same(6.0))
        .inner_margin(8.0)
        .show(ui, |ui| {
            kv_row(ui, "File Name", store.file_name(fid), ON_SURFACE);
            kv_row(ui, "Folder", store.dir_name(dir_id), ON_SURFACE);
            kv_row(ui, "File Size", &format_bytes(size), PRIMARY);
            if let Some(d) = meta {
                if d.meta.width > 0 {
                    kv_row(
                        ui,
                        "Resolution",
                        &format!("{} x {}", d.meta.width, d.meta.height),
                        ON_SURFACE,
                    );
                }
                if !d.meta.codec.is_empty() {
                    kv_row(ui, "Codec", &d.meta.codec, ON_SURFACE);
                }
                if !d.meta.camera.is_empty() {
                    kv_row(ui, "Camera", &d.meta.camera, ON_SURFACE);
                }
            }
        });
    ui.add_space(8.0);
    ui.label(
        RichText::new("SEMANTIC TAGS")
            .small()
            .strong()
            .color(ON_SURFACE_VARIANT),
    );
    let year = store.file_year.get(fid as usize).copied().unwrap_or(0);
    ui.horizontal_wrapped(|ui| {
        let folder_tag = store.dir_name(dir_id);
        if !folder_tag.is_empty() && tag_chip(ui, &format!("#{folder_tag}")) {
            act.filter_tag = Some(format!("dir:{folder_tag}"));
        }
        if year > 0 && tag_chip(ui, &format!("#{year}")) {
            act.filter_tag = Some(format!("date:{year}"));
        }
    });
}

pub fn render_inspector_actions(ui: &mut Ui, tr: &Strings, path: &Path, hash: &str) {
    let w = ui.available_width();
    if ui.add_sized([w, 28.0], Button::new(tr.ctx_open)).clicked() {
        open_path(path);
    }
    let rev = Button::new(RichText::new(tr.ctx_open_folder).color(SECONDARY));
    if let Some(p) = ui
        .add_sized([w, 28.0], rev)
        .clicked()
        .then(|| path.parent())
        .flatten()
    {
        open_path(p);
    }
    let copy_btn = Button::new(RichText::new("Copy Hash (BLAKE3)").color(TERTIARY))
        .min_size(Vec2::new(w, 28.0));
    if ui.add_enabled(!hash.is_empty(), copy_btn).clicked() {
        ui.ctx().copy_text(hash.to_owned());
    }
}

fn kv_row(ui: &mut Ui, key: &str, val: &str, col: Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(key).small().color(ON_SURFACE_VARIANT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(RichText::new(val).monospace().color(col)).truncate());
        });
    });
}

fn tag_chip(ui: &mut Ui, text: &str) -> bool {
    ui.add(
        Button::new(RichText::new(text).monospace().color(ON_SURFACE)).fill(SURFACE_CONTAINER),
    )
    .clicked()
}
