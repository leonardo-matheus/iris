//! # App Card Component
//! 
//! Componente de card para exibição de uma aplicação configurada.
//! Mostra nome, ícone, status e botões de ação.

use eframe::egui::{self, RichText};
use crate::core::AppConfig;
use crate::services::IconCache;
use crate::ui::theme::{self, ThemeColors, ThemeSpacing};

/// Resultado das interações com o card
pub struct CardActions {
    pub start_clicked: bool,
    pub stop_clicked: bool,
    pub restart_clicked: bool,
    pub edit_clicked: bool,
    pub delete_clicked: bool,
}

impl Default for CardActions {
    fn default() -> Self {
        Self {
            start_clicked: false,
            stop_clicked: false,
            restart_clicked: false,
            edit_clicked: false,
            delete_clicked: false,
        }
    }
}

/// Renderiza um card de aplicação.
/// 
/// # Argumentos
/// * `ui` - Contexto de UI do egui
/// * `app` - Configuração da aplicação
/// * `is_running` - Se a aplicação está executando
/// * `is_loading` - Se a aplicação está iniciando
/// * `icon_cache` - Cache de ícones para renderização
/// 
/// # Retorno
/// `CardActions` com os botões que foram clicados
pub fn render_app_card(
    ui: &mut egui::Ui,
    app: &AppConfig,
    is_running: bool,
    is_loading: bool,
    icon_cache: &mut IconCache,
) -> CardActions {
    let mut actions = CardActions::default();

    let (bg_color, border_color, glow_color) = theme::get_card_colors(is_running, is_loading);
    let card_width = ThemeSpacing::CARD_WIDTH;
    let card_height = ThemeSpacing::CARD_HEIGHT;
    
    let response = theme::card_frame(bg_color, border_color, glow_color)
        .show(ui, |ui| {
            ui.set_width(card_width);
            ui.set_min_height(card_height);

            ui.vertical(|ui| {
                ui.set_min_height(card_height);
                
                // Header do card
                render_card_header(ui, app, is_running, is_loading, icon_cache, &mut actions);

                ui.add_space(12.0);

                // Nome da aplicação
                ui.label(
                    RichText::new(&app.name)
                        .size(17.0)
                        .strong()
                        .color(ThemeColors::TEXT_PRIMARY),
                );

                ui.add_space(4.0);

                // Status badge
                render_status_badge(ui, card_width, is_running, is_loading);

                ui.add_space(8.0);

                // Info do projeto
                render_project_info(ui, app, card_width);

                // Preencher espaço restante
                ui.add_space(ui.available_height() - 46.0);

                // Botões de ação
                render_action_buttons(ui, card_width, is_running, is_loading, &mut actions);
            });
        });

    if response.response.hovered() {
        ui.ctx().request_repaint();
    }

    actions
}

fn render_card_header(
    ui: &mut egui::Ui,
    app: &AppConfig,
    is_running: bool,
    is_loading: bool,
    icon_cache: &mut IconCache,
    actions: &mut CardActions,
) {
    ui.horizontal(|ui| {
        // Ícone da aplicação
        egui::Frame::none()
            .fill(ThemeColors::BG_ICON)
            .rounding(10.0)
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(44.0, 44.0));
                if !app.icon_emoji.is_empty() {
                    if let Some(texture) = icon_cache.get_or_load(ui.ctx(), &app.icon_emoji) {
                        ui.image(egui::load::SizedTexture::new(
                            texture.id(),
                            egui::vec2(28.0, 28.0),
                        ));
                    } else {
                        ui.label(RichText::new("🚀").size(24.0));
                    }
                } else {
                    ui.label(RichText::new("🚀").size(24.0));
                }
            });
        
        // Botões de edição/exclusão
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(2.0, 0.0);
                
                if !is_running && !is_loading {
                    let del_btn = egui::Button::new(
                        RichText::new("x").size(14.0).color(ThemeColors::BTN_DELETE)
                    )
                    .fill(ThemeColors::BTN_DELETE_BG)
                    .rounding(6.0)
                    .min_size(egui::vec2(24.0, 24.0));
                    if ui.add(del_btn).on_hover_text("Deletar").clicked() {
                        actions.delete_clicked = true;
                    }
                }
                if !is_loading {
                    let edit_btn = egui::Button::new(
                        RichText::new("...").size(12.0).color(ThemeColors::TEXT_SECONDARY)
                    )
                    .fill(egui::Color32::from_rgb(50, 50, 58))
                    .rounding(6.0)
                    .min_size(egui::vec2(24.0, 24.0));
                    if ui.add(edit_btn).on_hover_text("Editar").clicked() {
                        actions.edit_clicked = true;
                    }
                }
            });
        });
    });
}

fn render_status_badge(ui: &mut egui::Ui, card_width: f32, is_running: bool, is_loading: bool) {
    ui.allocate_ui_with_layout(
        egui::vec2(card_width, 18.0),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            if is_running {
                egui::Frame::none()
                    .fill(ThemeColors::RUNNING_BADGE_BG)
                    .rounding(4.0)
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("▶ Executando")
                                .size(10.0)
                                .color(ThemeColors::RUNNING_BORDER),
                        );
                    });
            } else if is_loading {
                egui::Frame::none()
                    .fill(egui::Color32::from_rgb(40, 40, 60))
                    .rounding(4.0)
                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("⏳ Iniciando...")
                                .size(10.0)
                                .color(ThemeColors::LOADING_BORDER),
                        );
                    });
            }
        },
    );
}

fn render_project_info(ui: &mut egui::Ui, app: &AppConfig, card_width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(card_width, 16.0),
        egui::Layout::left_to_right(egui::Align::Min),
        |ui| {
            if !app.working_dir.is_empty() {
                ui.label(
                    RichText::new(format!("📂 {}", crate::utils::truncate_path(&app.working_dir, 28)))
                        .size(11.0)
                        .color(egui::Color32::from_rgb(120, 120, 130)),
                );
            }
        },
    );

    ui.label(
        RichText::new(format!("⚡ {} comando(s)", app.commands.len()))
            .size(11.0)
            .color(ThemeColors::TEXT_MUTED),
    );
}

fn render_action_buttons(
    ui: &mut egui::Ui,
    card_width: f32,
    is_running: bool,
    is_loading: bool,
    actions: &mut CardActions,
) {
    let button_width = card_width - 36.0;
    
    if is_loading {
        let button = egui::Button::new(
            RichText::new("⏳ Iniciando...")
                .size(13.0)
                .color(egui::Color32::from_rgb(150, 150, 160)),
        )
        .fill(egui::Color32::from_rgb(50, 50, 58))
        .rounding(ThemeSpacing::BUTTON_ROUNDING)
        .min_size(egui::vec2(button_width, ThemeSpacing::BUTTON_HEIGHT));
        ui.add_enabled(false, button);
    } else if is_running {
        ui.horizontal(|ui| {
            let btn_width = (button_width - 8.0) / 2.0;
            
            let stop_button = theme::action_button("■ Stop", ThemeColors::BTN_DANGER)
                .min_size(egui::vec2(btn_width, ThemeSpacing::BUTTON_HEIGHT));

            if ui.add(stop_button).clicked() {
                actions.stop_clicked = true;
            }

            let restart_button = theme::action_button("↻ Restart", ThemeColors::BTN_WARNING)
                .min_size(egui::vec2(btn_width, ThemeSpacing::BUTTON_HEIGHT));

            if ui.add(restart_button).clicked() {
                actions.restart_clicked = true;
            }
        });
    } else {
        let button = egui::Button::new(
            RichText::new("▶  Executar")
                .size(13.0)
                .color(ThemeColors::TEXT_PRIMARY),
        )
        .fill(ThemeColors::BTN_PRIMARY)
        .rounding(ThemeSpacing::BUTTON_ROUNDING)
        .min_size(egui::vec2(button_width, ThemeSpacing::BUTTON_HEIGHT));

        if ui.add(button).clicked() {
            actions.start_clicked = true;
        }
    }
}
