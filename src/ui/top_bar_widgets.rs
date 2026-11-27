use egui::{Frame, RichText, Rounding, Stroke, TextEdit, Ui};

use crate::i18n::Strings;
use crate::index::store::ArchiveStore;
use crate::ui::theme::{
    ON_SURFACE_VARIANT, OUTLINE_VARIANT, SECONDARY, SURFACE_CONTAINER, SURFACE_LOW,
};
use crate::ui::top_bar::TopBarAction;

pub fn render_breadcrumbs(
    ui: &mut Ui,
    tr: &Strings,
    store: Option<&ArchiveStore>,
    selected_dir: Option<u32>,
    act: &mut TopBarAction,
) {
    Frame::none()
        .fill(SURFACE_LOW)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(8.0, 4.0))
        .show(ui, |ui| {
            ui.set_min_width(170.0);
            ui.set_max_width(270.0);
            ui.horizontal(|ui| {
                let Some(st) = store else {
                    ui.label(RichText::new(tr.address_placeholder).color(ON_SURFACE_VARIANT));
                    return;
                };
                let chain = dir_chain(st, selected_dir.unwrap_or(0));
                for (i, dir_id) in chain.into_iter().enumerate() {
                    if i > 0 {
                        ui.label(RichText::new(">").small().color(ON_SURFACE_VARIANT));
                    }
                    let target = (dir_id != 0).then_some(dir_id);
                    if ui
                        .selectable_label(selected_dir == target, st.dir_name(dir_id))
                        .clicked()
                    {
                        act.select_dir = Some(target);
                    }
                }
            });
        });
}

fn dir_chain(st: &ArchiveStore, start: u32) -> Vec<u32> {
    let mut chain = Vec::new();
    let mut cur = start;
    while cur != 0 {
        chain.push(cur);
        cur = st.dir_parent.get(cur as usize).copied().unwrap_or(0);
    }
    chain.push(0);
    chain.reverse();
    chain
}

pub fn render_search_capsule(
    ui: &mut Ui,
    tr: &Strings,
    query: &mut String,
    elapsed_us: u64,
    act: &mut TopBarAction,
) {
    Frame::none()
        .fill(SURFACE_LOW)
        .stroke(Stroke::new(1.0_f32, OUTLINE_VARIANT))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(8.0, 3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let resp = ui.add_sized(
                    [230.0, 22.0],
                    TextEdit::singleline(query)
                        .frame(false)
                        .hint_text(tr.search_hint),
                );
                if resp.changed() {
                    act.query_changed = true;
                }
                if !query.is_empty() && ui.small_button("x").clicked() {
                    query.clear();
                    act.query_changed = true;
                }
                let ms = elapsed_us as f64 / 1000.0;
                Frame::none()
                    .fill(SURFACE_CONTAINER)
                    .rounding(Rounding::same(4.0))
                    .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("{ms:.1} ms"))
                                .monospace()
                                .color(SECONDARY),
                        );
                    });
            });
        });
}
