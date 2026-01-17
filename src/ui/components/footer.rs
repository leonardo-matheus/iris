//! # Footer Component
//! 
//! Componente de rodapé da aplicação.
//! Mostra estatísticas e informações sobre a aplicação.

use eframe::egui::{self, RichText};
use crate::ui::theme::ThemeColors;

/// Renderiza o rodapé da aplicação.
/// 
/// # Argumentos
/// * `ui` - Contexto de UI do egui
/// * `app_count` - Número total de aplicações
/// * `running_count` - Número de aplicações em execução
pub fn render_footer(ui: &mut egui::Ui, app_count: usize, running_count: usize) {
    ui.horizontal(|ui| {
        // Badge de contagem de apps
        egui::Frame::none()
            .fill(egui::Color32::from_rgb(38, 38, 45))
            .rounding(6.0)
            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(format!("📦 {} apps", app_count))
                        .size(12.0)
                        .color(ThemeColors::TEXT_SECONDARY),
                );
            });
        
        ui.add_space(12.0);
        
        // Badge de aplicações em execução
        if running_count > 0 {
            egui::Frame::none()
                .fill(ThemeColors::RUNNING_BADGE_BG)
                .rounding(6.0)
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("● {} executando", running_count))
                            .size(12.0)
                            .color(ThemeColors::RUNNING_TEXT),
                    );
                });
        }
        
        // Informações da versão (alinhado à direita)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new("Feito com ❤️ em Rust")
                    .size(11.0)
                    .color(ThemeColors::TEXT_SUBTLE),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("•")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(60, 60, 70)),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("v1.0.0")
                    .size(11.0)
                    .color(ThemeColors::TEXT_SUBTLE),
            );
        });
    });
}
