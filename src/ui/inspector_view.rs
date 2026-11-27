use std::path::Path;

use egui::{Button, Color32, Frame, RichText, Rounding, Ui, Vec2};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::format_bytes;
use crate::ui::inspector::InspectorAction;
use crate::ui::inspector_loader::InspectorData;
use crate::ui::os_open::open_path;
use crate::ui::theme::{
    ON_SURFACE, ON_SURFACE_VARIANT, PRIMARY, SECONDARY, SURFACE_CONTAINER, SURFACE_LOW,
    TERTIARY,
};

pub fn render_meta_and_tags(
    ui: &mut Ui,
    store: &ArchiveStore,
    fid: u32,
    meta: Option<&InspectorData>,
    vec_indexed: bool,
    act: &mut InspectorAction,
) {
    let dir_id = store.file_dir.get(fid as usize).copied().unwrap_or(0);
    let size = meta.map_or_else(
        || store.file_size.get(fid as usize).copied().unwrap_or(0),
        |d| d.file_size,
    );
    let year = meta
        .map(|d| d.meta.year)
        .filter(|&y| y > 0)
        .unwrap_or_else(|| store.file_year.get(fid as usize).copied().unwrap_or(0));

    Frame::none()
        .fill(SURFACE_LOW)
        .rounding(Rounding::same(6.0))
        .inner_margin(8.0)
        .show(ui, |ui| {
            kv_row(ui, "File Name", store.file_name(fid), ON_SURFACE);
            kv_row(ui, "Folder", store.dir_name(dir_id), ON_SURFACE);
            if let Some(d) = meta.filter(|d| d.meta.width > 0) {
                let res = format!("{} x {}", d.meta.width, d.meta.height);
                kv_row(ui, "Resolution", &res, ON_SURFACE);
            }
            kv_row(ui, "Camera / Codec", camera_or_codec(meta), ON_SURFACE);
            let date_lbl = if year > 0 {
                year.to_string()
            } else {
                "—".to_owned()
            };
            kv_row(ui, "Date Taken", &date_lbl, ON_SURFACE);
            kv_row(ui, "File Size", &format_bytes(size), PRIMARY);
            let (vtxt, vcol) = if vec_indexed {
                ("Indexed (GPU)", SECONDARY)
            } else {
                ("Not indexed", ON_SURFACE_VARIANT)
            };
            kv_row(ui, "AI Vector", vtxt, vcol);
        });
    render_tags(ui, store.dir_name(dir_id), year, act);
}

fn camera_or_codec(meta: Option<&InspectorData>) -> &str {
    meta.map_or("—", |d| {
        if !d.meta.camera.is_empty() {
            &d.meta.camera
        } else if !d.meta.codec.is_empty() {
            &d.meta.codec
        } else {
            "—"
        }
    })
}

fn render_tags(ui: &mut Ui, folder_tag: &str, year: u16, act: &mut InspectorAction) {
    ui.add_space(8.0);
    ui.label(
        RichText::new("SEMANTIC TAGS")
            .small()
            .strong()
            .color(ON_SURFACE_VARIANT),
    );
    ui.horizontal_wrapped(|ui| {
        if !folder_tag.is_empty() && tag_chip(ui, &format!("#{folder_tag}")) {
            act.filter_tag = Some(format!("dir:{folder_tag}"));
        }
        if year > 0 && tag_chip(ui, &format!("#{year}")) {
            act.filter_tag = Some(format!("year:{year}"));
        }
    });
}

pub fn render_inspector_actions(
    ui: &mut Ui,
    tr: &Strings,
    fid: u32,
    path: &Path,
    hash: &str,
    copied_id: &mut Option<u32>,
) {
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
    let copied = *copied_id == Some(fid);
    let lbl = if copied {
        "Copied BLAKE3!"
    } else {
        "Copy Hash (BLAKE3)"
    };
    let col = if copied { SECONDARY } else { TERTIARY };
    let copy_btn = Button::new(RichText::new(lbl).color(col)).min_size(Vec2::new(w, 28.0));
    if ui.add_enabled(!hash.is_empty(), copy_btn).clicked() {
        ui.ctx().copy_text(hash.to_owned());
        *copied_id = Some(fid);
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
