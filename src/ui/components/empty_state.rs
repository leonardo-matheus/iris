//! # Empty State Component
//! 
//! Componente exibido quando não há aplicações configuradas.
//! Mostra uma mensagem de boas-vindas e botão para criar primeira app.

use eframe::egui::{self, RichText};
use crate::ui::theme::ThemeColors;

/// Renderiza o estado vazio (quando não há aplicações).
/// 
/// # Argumentos
/// * `ui` - Contexto de UI do egui
/// 
/// # Retorno
/// `true` se o botão de criar app foi clicado
pub fn render_empty_state(ui: &mut egui::Ui) -> bool {
    let mut create_clicked = false;
    
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        
        // Ícone grande com container
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(30, 30, 38))
            .rounding(24.0)
            .inner_margin(egui::Margin::same(32.0))
            .show(ui, |ui| {
                ui.label(RichText::new("🌈").size(56.0));
            });
        
        ui.add_space(28.0);
        
        ui.label(
            RichText::new("Bem-vindo ao Iris!")
                .size(28.0)
                .strong()
                .color(ThemeColors::TEXT_PRIMARY),
        );
        
        ui.add_space(8.0);
        
        ui.label(
            RichText::new("Seu hub de aplicações está vazio")
                .size(16.0)
                .color(egui::Color32::from_rgb(140, 140, 150)),
        );
        
        ui.add_space(24.0);
        
        // Botão de criar primeira app
        let btn = egui::Button::new(
            RichText::new("➕  Criar primeira aplicação")
                .size(15.0)
                .color(ThemeColors::TEXT_PRIMARY),
        )
        .fill(ThemeColors::BTN_PRIMARY)
        .rounding(12.0)
        .min_size(egui::vec2(220.0, 44.0));
        
        if ui.add(btn).clicked() {
            create_clicked = true;
        }
        
        ui.add_space(16.0);
        
        ui.label(
            RichText::new("Configure comandos, escolha um ícone e execute com um clique!")
                .size(12.0)
                .color(ThemeColors::TEXT_MUTED),
        );
    });
    
    create_clicked
}

/// Renderiza o estado de busca vazia (nenhum resultado encontrado).
/// 
/// # Argumentos
/// * `ui` - Contexto de UI do egui
/// * `search_term` - Termo de busca que não retornou resultados
pub fn render_no_results(ui: &mut egui::Ui, search_term: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(60.0);
        
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(28, 28, 35))
            .rounding(16.0)
            .inner_margin(egui::Margin::same(24.0))
            .show(ui, |ui| {
                ui.label(RichText::new("🔍").size(32.0));
                ui.add_space(12.0);
                ui.label(
                    RichText::new("Nenhuma aplicação encontrada")
                        .size(16.0)
                        .color(ThemeColors::TEXT_SECONDARY),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("Nenhum resultado para \"{}\".", search_term))
                        .size(13.0)
                        .color(ThemeColors::TEXT_MUTED),
                );
            });
    });
}
