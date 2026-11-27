use egui::{Color32, Context, FontId, Rounding, Stroke, TextStyle, Visuals};

pub const BG_CANVAS: Color32 = Color32::from_rgb(25, 25, 25);
pub const BG_PANEL: Color32 = Color32::from_rgb(32, 32, 32);
pub const BG_SURFACE: Color32 = Color32::from_rgb(40, 40, 40);
pub const BG_HOVER: Color32 = Color32::from_rgb(49, 49, 49);
pub const BG_SELECTED: Color32 = Color32::from_rgb(44, 62, 82);
pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(54, 54, 54);
pub const ACCENT: Color32 = Color32::from_rgb(76, 194, 255);
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 243, 243);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(157, 157, 157);

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_PANEL;
    visuals.extreme_bg_color = BG_CANVAS;
    visuals.faint_bg_color = BG_SURFACE;
    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.selection.bg_fill = BG_SELECTED;
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.window_rounding = Rounding::same(6.0);
    visuals.window_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);

    style_widgets(&mut visuals);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(15.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.5));
    ctx.set_style(style);
}

fn style_widgets(visuals: &mut Visuals) {
    let r = Rounding::same(4.0);
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.noninteractive.rounding = r;

    visuals.widgets.inactive.bg_fill = BG_SURFACE;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.inactive.rounding = r;

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER_SUBTLE);
    visuals.widgets.hovered.rounding = r;

    visuals.widgets.active.bg_fill = BG_SELECTED;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, ACCENT);
    visuals.widgets.active.rounding = r;
}
