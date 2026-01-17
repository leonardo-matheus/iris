//! # Theme Module
//! 
//! Configurações de tema e estilo visual da aplicação Iris.
//! Define cores, espaçamentos e estilos visuais.

use eframe::egui::{self, Color32, Rounding, Margin, Stroke};

/// Cores do tema escuro da aplicação
pub struct ThemeColors;

impl ThemeColors {
    // Cores de fundo
    pub const BG_DARK: Color32 = Color32::from_rgb(18, 18, 22);
    pub const BG_HEADER: Color32 = Color32::from_rgb(22, 22, 26);
    pub const BG_CARD: Color32 = Color32::from_rgb(32, 32, 38);
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(38, 38, 45);
    pub const BG_INPUT: Color32 = Color32::from_rgb(38, 38, 45);
    pub const BG_ICON: Color32 = Color32::from_rgb(45, 45, 52);
    
    // Cores de estado - Executando
    pub const RUNNING_BG: Color32 = Color32::from_rgb(25, 35, 30);
    pub const RUNNING_BORDER: Color32 = Color32::from_rgb(34, 197, 94);
    pub const RUNNING_BADGE_BG: Color32 = Color32::from_rgb(34, 55, 40);
    pub const RUNNING_TEXT: Color32 = Color32::from_rgb(80, 220, 120);
    
    // Cores de estado - Loading
    pub const LOADING_BG: Color32 = Color32::from_rgb(25, 30, 40);
    pub const LOADING_BORDER: Color32 = Color32::from_rgb(99, 102, 241);
    
    // Cores de botões
    pub const BTN_PRIMARY: Color32 = Color32::from_rgb(99, 102, 241);  // Indigo
    pub const BTN_DANGER: Color32 = Color32::from_rgb(239, 68, 68);   // Vermelho
    pub const BTN_WARNING: Color32 = Color32::from_rgb(245, 158, 11); // Amarelo
    pub const BTN_DELETE: Color32 = Color32::from_rgb(180, 80, 80);
    pub const BTN_DELETE_BG: Color32 = Color32::from_rgb(60, 40, 40);
    
    // Cores de texto
    pub const TEXT_PRIMARY: Color32 = Color32::WHITE;
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 160, 170);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(100, 100, 110);
    pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(80, 80, 90);
    
    // Cores de borda
    pub const BORDER_DEFAULT: Color32 = Color32::from_rgb(55, 55, 62);
}

/// Configurações de espaçamento
pub struct ThemeSpacing;

impl ThemeSpacing {
    pub const CARD_WIDTH: f32 = 250.0;
    pub const CARD_HEIGHT: f32 = 240.0;
    pub const CARD_PADDING: f32 = 18.0;
    pub const CARD_ROUNDING: f32 = 16.0;
    pub const CARD_GRID_SPACING: f32 = 16.0;
    
    pub const BUTTON_ROUNDING: f32 = 10.0;
    pub const BUTTON_HEIGHT: f32 = 36.0;
    
    pub const ICON_SIZE: f32 = 32.0;
    pub const ICON_CONTAINER_SIZE: f32 = 44.0;
}

/// Aplica o tema visual ao contexto egui
pub fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    visuals.window_rounding = Rounding::same(12.0);
    visuals.window_shadow = egui::epaint::Shadow {
        offset: egui::vec2(0.0, 8.0),
        blur: 24.0,
        spread: 0.0,
        color: Color32::from_black_alpha(100),
    };
    visuals.popup_shadow = visuals.window_shadow;
    
    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 28, 32);
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 45, 50);
    visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(45, 45, 50);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 68);
    visuals.widgets.active.bg_fill = Color32::from_rgb(70, 70, 78);
    visuals.selection.bg_fill = Color32::from_rgb(0, 120, 215);
    visuals.extreme_bg_color = Color32::from_rgb(20, 20, 24);
    visuals.faint_bg_color = Color32::from_rgb(35, 35, 40);
    
    ctx.set_visuals(visuals);
    
    // Configurar estilo
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = Margin::same(16.0);
    style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
    style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
    style.visuals.widgets.hovered.rounding = Rounding::same(8.0);
    style.visuals.widgets.active.rounding = Rounding::same(8.0);
    ctx.set_style(style);
}

/// Retorna as cores do card baseadas no estado
pub fn get_card_colors(is_running: bool, is_loading: bool) -> (Color32, Color32, Color32) {
    if is_running {
        (
            ThemeColors::RUNNING_BG,
            ThemeColors::RUNNING_BORDER,
            Color32::from_rgba_unmultiplied(34, 197, 94, 30),
        )
    } else if is_loading {
        (
            ThemeColors::LOADING_BG,
            ThemeColors::LOADING_BORDER,
            Color32::from_rgba_unmultiplied(99, 102, 241, 30),
        )
    } else {
        (
            ThemeColors::BG_CARD,
            ThemeColors::BORDER_DEFAULT,
            Color32::TRANSPARENT,
        )
    }
}

/// Cria um frame estilizado para cards
pub fn card_frame(bg_color: Color32, border_color: Color32, glow_color: Color32) -> egui::Frame {
    egui::Frame::none()
        .fill(bg_color)
        .rounding(ThemeSpacing::CARD_ROUNDING)
        .inner_margin(Margin::same(ThemeSpacing::CARD_PADDING))
        .stroke(Stroke::new(1.5, border_color))
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: glow_color,
        })
}

/// Cria um botão primário estilizado
pub fn primary_button(text: &str) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(text)
            .size(14.0)
            .color(ThemeColors::TEXT_PRIMARY),
    )
    .fill(ThemeColors::BTN_PRIMARY)
    .rounding(ThemeSpacing::BUTTON_ROUNDING)
}

/// Cria um botão de ação (danger/warning)
pub fn action_button(text: &str, color: Color32) -> egui::Button<'_> {
    egui::Button::new(
        egui::RichText::new(text)
            .size(12.0)
            .color(ThemeColors::TEXT_PRIMARY),
    )
    .fill(color)
    .rounding(ThemeSpacing::BUTTON_ROUNDING)
}
