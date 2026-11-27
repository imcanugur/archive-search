use egui::{Color32, Context, FontId, Rounding, Stroke, TextStyle, Visuals};

pub const SURFACE: Color32 = Color32::from_rgb(248, 249, 255);
pub const SURFACE_LOWEST: Color32 = Color32::from_rgb(255, 255, 255);
pub const SURFACE_LOW: Color32 = Color32::from_rgb(239, 244, 255);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(229, 238, 255);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(220, 233, 255);
pub const PRIMARY: Color32 = Color32::from_rgb(0, 103, 192);
pub const PRIMARY_DARK: Color32 = Color32::from_rgb(0, 79, 150);
pub const PRIMARY_FIXED: Color32 = Color32::from_rgb(213, 227, 255);
pub const SECONDARY: Color32 = Color32::from_rgb(0, 106, 99);
pub const SECONDARY_BG: Color32 = Color32::from_rgb(236, 253, 245);
pub const TERTIARY: Color32 = Color32::from_rgb(131, 57, 0);
pub const TERTIARY_BG: Color32 = Color32::from_rgb(255, 245, 235);
pub const ON_SURFACE: Color32 = Color32::from_rgb(11, 28, 48);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(65, 71, 82);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(193, 198, 212);
pub const INVERSE_SURFACE: Color32 = Color32::from_rgb(33, 49, 69);
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
    visuals.faint_bg_color = SURFACE_LOW;
    visuals.override_text_color = Some(ON_SURFACE);
    visuals.selection.bg_fill = PRIMARY_FIXED;
    visuals.selection.stroke = Stroke::new(1.0_f32, PRIMARY);
    visuals.window_rounding = Rounding::same(8.0);
    visuals.window_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);

    style_widgets(&mut visuals);
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
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

    visuals.widgets.inactive.bg_fill = SURFACE_LOW;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, OUTLINE_VARIANT);
    visuals.widgets.inactive.rounding = r;

    visuals.widgets.hovered.bg_fill = SURFACE_CONTAINER;
    visuals.widgets.hovered.weak_bg_fill = SURFACE_CONTAINER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, PRIMARY);
    visuals.widgets.hovered.rounding = r;

    visuals.widgets.active.bg_fill = SURFACE_HIGH;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, PRIMARY);
    visuals.widgets.active.rounding = r;
}
