use egui::{Color32, Context, FontId, Rounding, Stroke, TextStyle, Visuals};

pub const SURFACE: Color32 = Color32::from_rgb(243, 245, 248);
pub const SURFACE_LOWEST: Color32 = Color32::from_rgb(255, 255, 255);
pub const SURFACE_LOW: Color32 = Color32::from_rgb(239, 244, 255);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(229, 238, 255);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(235, 243, 252);
pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(248, 250, 252);
pub const PRIMARY: Color32 = Color32::from_rgb(0, 103, 192);
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0, 79, 150);
pub const PRIMARY_FIXED: Color32 = Color32::from_rgb(213, 227, 255);
pub const SECONDARY: Color32 = Color32::from_rgb(15, 118, 110);
pub const SECONDARY_BG: Color32 = Color32::from_rgb(236, 253, 245);
pub const TERTIARY: Color32 = Color32::from_rgb(180, 83, 9);
pub const TERTIARY_BG: Color32 = Color32::from_rgb(255, 251, 235);
pub const ON_SURFACE: Color32 = Color32::from_rgb(15, 23, 42);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(71, 85, 105);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(226, 232, 240);
pub const BORDER_STRONG: Color32 = Color32::from_rgb(203, 213, 225);
pub const INVERSE_SURFACE: Color32 = Color32::from_rgb(22, 33, 51);
pub const INVERSE_TEXT: Color32 = Color32::from_rgb(234, 241, 255);

pub const BG_SELECTED: Color32 = SURFACE_HIGH;
pub const ACCENT: Color32 = PRIMARY;
pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
pub const TEXT_MUTED: Color32 = ON_SURFACE_VARIANT;

pub fn apply_theme(ctx: &Context) {
    let mut visuals = Visuals::light();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE_LOWEST;
    visuals.extreme_bg_color = SURFACE_LOW;
    visuals.faint_bg_color = BG_CARD_HOVER;
    visuals.override_text_color = Some(ON_SURFACE);
    visuals.selection.bg_fill = PRIMARY_FIXED;
    visuals.selection.stroke = Stroke::new(1.0_f32, PRIMARY);
    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);

    style_widgets(&mut visuals);
    ctx.set_visuals(visuals);
    apply_typography(ctx);
}

fn apply_typography(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.scroll = egui::style::ScrollStyle::thin();
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(15.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(12.5));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.5));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(11.5));
    ctx.set_style(style);
}

fn style_widgets(visuals: &mut Visuals) {
    let r = Rounding::same(6.0);
    visuals.widgets.noninteractive.bg_fill = SURFACE_LOWEST;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    visuals.widgets.noninteractive.rounding = r;

    visuals.widgets.inactive.bg_fill = SURFACE_LOWEST;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    visuals.widgets.inactive.rounding = r;

    visuals.widgets.hovered.bg_fill = BG_CARD_HOVER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_LOW;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, BORDER_STRONG);
    visuals.widgets.hovered.rounding = r;

    visuals.widgets.active.bg_fill = SURFACE_HIGH;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, PRIMARY);
    visuals.widgets.active.rounding = r;
}
