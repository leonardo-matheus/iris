//! # Header Component
//! 
//! Componente de cabeçalho da aplicação.
//! Inclui logo, título, campo de busca e botão de nova aplicação.

use eframe::egui::{self, RichText};
use crate::ui::theme::ThemeColors;

/// Resultado das interações com o header
pub struct HeaderActions {
    pub add_app_clicked: bool,
    pub export_clicked: bool,
    pub import_clicked: bool,
}

impl Default for HeaderActions {
    fn default() -> Self {
        Self {
            add_app_clicked: false,
            export_clicked: false,
            import_clicked: false,
        }
    }
}

/// Renderiza o cabeçalho da aplicação.
/// 
/// # Argumentos
/// * `ui` - Contexto de UI do egui
/// * `search_filter` - Referência mutável ao filtro de busca
/// 
/// # Retorno
/// `HeaderActions` com os botões que foram clicados
pub fn render_header(ui: &mut egui::Ui, search_filter: &mut String) -> HeaderActions {
    let mut actions = HeaderActions::default();
    
    ui.horizontal(|ui| {
        // Logo e título
        ui.heading(RichText::new("🌈").size(32.0));
        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.add_space(2.0);
            ui.label(
                RichText::new("Iris")
                    .size(24.0)
                    .strong()
                    .color(ThemeColors::TEXT_PRIMARY),
            );
            ui.label(
                RichText::new("Mensageira dos Devs")
                    .size(11.0)
                    .color(egui::Color32::from_rgb(140, 140, 150)),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Botão Nova Aplicação
            let add_btn = egui::Button::new(
                RichText::new("➕  Nova Aplicação")
                    .size(14.0)
                    .color(ThemeColors::TEXT_PRIMARY),
            )
            .fill(ThemeColors::BTN_PRIMARY)
            .rounding(10.0)
            .min_size(egui::vec2(140.0, 38.0));
            
            if ui.add(add_btn).clicked() {
                actions.add_app_clicked = true;
            }

            ui.add_space(8.0);

            // Menu de configurações
            ui.menu_button(
                RichText::new("⚙")
                    .size(16.0)
                    .color(ThemeColors::TEXT_SECONDARY),
                |ui| {
                    ui.set_min_width(160.0);
                    
                    if ui.button("📤  Exportar configurações").clicked() {
                        actions.export_clicked = true;
                        ui.close_menu();
                    }
                    
                    if ui.button("📥  Importar configurações").clicked() {
                        actions.import_clicked = true;
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    ui.label(
                        RichText::new("Compartilhe suas configurações!")
                            .size(10.0)
                            .color(egui::Color32::from_rgb(120, 120, 130))
                    );
                }
            );

            ui.add_space(16.0);

            // Campo de busca
            egui::Frame::none()
                .fill(ThemeColors::BG_INPUT)
                .rounding(10.0)
                .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("🔍")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(120, 120, 130)),
                        );
                        ui.add(
                            egui::TextEdit::singleline(search_filter)
                                .desired_width(180.0)
                                .frame(false)
                                .hint_text(
                                    RichText::new("Buscar aplicações...")
                                        .color(ThemeColors::TEXT_MUTED)
                                ),
                        );
                    });
                });
        });
    });
    
    actions
}
