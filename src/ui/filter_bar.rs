use egui::{Button, RichText, Rounding, Ui};

use crate::ui::theme::{
    ON_SURFACE, PRIMARY, PRIMARY_FIXED, SURFACE_LOW, TERTIARY, TERTIARY_BG,
};

#[derive(Default)]
pub struct FilterBarAction {
    pub query_changed: bool,
    pub clear_all: bool,
}

const TYPE_CHIPS: [(&str, &str); 3] = [
    ("Photos", "type:photo"),
    ("Videos", "type:video"),
    ("RAW", "type:raw"),
];

const SIZE_CHIPS: [(&str, &str); 4] = [
    (">1MB", "size:>1mb"),
    (">10MB", "size:>10mb"),
    (">50MB", "size:>50mb"),
    ("<1MB", "size:<1mb"),
];

const YEAR_CHIPS: [(&str, &str); 4] = [
    ("2025", "year:2025"),
    ("2024", "year:2024"),
    ("2023", "year:2023"),
    ("2020..2023", "year:2020..2023"),
];

const SYNTAX_HELP: &str = "Advanced Filter Syntax:\n\
• type:photo | type:video | type:raw | type:other\n\
• ext:jpg,png,heic  (OR extensions)   • -ext:tmp  (exclude)\n\
• size:>10mb | size:<1.5gb | size:5mb..50mb\n\
• year:2024 | year:>=2022 | year:2020..2024\n\
• dir:antalya  (folder)   • -dir:cache  (exclude folder)\n\
• -thumb  (exclude word)   • \"yaz tatili\"  (exact phrase)";

pub fn render_filter_bar(
    ui: &mut Ui,
    query: &mut String,
    has_dir: bool,
    size_progress: Option<(u32, u32)>,
) -> FilterBarAction {
    let mut act = FilterBarAction::default();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
        let no_type = !has_prefix(query, "type:");
        if chip(ui, "All", no_type) {
            remove_prefix(query, "type:");
            act.query_changed = true;
        }
        render_chip_group(ui, query, "type:", &TYPE_CHIPS, &mut act);
        ui.separator();
        render_chip_group(ui, query, "size:", &SIZE_CHIPS, &mut act);
        if let Some((done, total)) = size_progress {
            let pct = (u64::from(done) * 100) / u64::from(total.max(1));
            let lbl = format!("(sizing {pct}% — {done}/{total})");
            ui.label(RichText::new(lbl).small().color(TERTIARY));
        }
        ui.separator();
        render_chip_group(ui, query, "year:", &YEAR_CHIPS, &mut act);

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new("? Syntax").small().color(PRIMARY))
                .on_hover_text(SYNTAX_HELP);
            if !query.is_empty() || has_dir {
                let clr = Button::new(RichText::new("Clear Filters").small().color(TERTIARY))
                    .fill(TERTIARY_BG)
                    .rounding(Rounding::same(10.0));
                if ui.add(clr).clicked() {
                    act.clear_all = true;
                }
            }
        });
    });
    act
}

fn render_chip_group(
    ui: &mut Ui,
    query: &mut String,
    prefix: &str,
    items: &[(&str, &str)],
    act: &mut FilterBarAction,
) {
    for &(label, token) in items {
        let active = has_token(query, token);
        if chip(ui, label, active) {
            toggle_exclusive_token(query, prefix, token);
            act.query_changed = true;
        }
    }
}

fn chip(ui: &mut Ui, label: &str, active: bool) -> bool {
    let fill = if active { PRIMARY_FIXED } else { SURFACE_LOW };
    let text = if active {
        RichText::new(label).small().strong().color(PRIMARY)
    } else {
        RichText::new(label).small().color(ON_SURFACE)
    };
    ui.add(Button::new(text).fill(fill).rounding(Rounding::same(10.0)))
        .clicked()
}

fn has_token(query: &str, token: &str) -> bool {
    query
        .split_whitespace()
        .any(|t| t.eq_ignore_ascii_case(token))
}

fn has_prefix(query: &str, prefix: &str) -> bool {
    let p_len = prefix.len();
    query.split_whitespace().any(|t| {
        t.get(..p_len)
            .is_some_and(|h| h.eq_ignore_ascii_case(prefix))
    })
}

fn remove_prefix(query: &mut String, prefix: &str) {
    let p_len = prefix.len();
    let kept: Vec<&str> = query
        .split_whitespace()
        .filter(|t| {
            !t.get(..p_len)
                .is_some_and(|h| h.eq_ignore_ascii_case(prefix))
        })
        .collect();
    *query = kept.join(" ");
}

fn toggle_exclusive_token(query: &mut String, prefix: &str, token: &str) {
    let was_active = has_token(query, token);
    remove_prefix(query, prefix);
    if !was_active {
        if !query.is_empty() {
            query.push(' ');
        }
        query.push_str(token);
    }
}
