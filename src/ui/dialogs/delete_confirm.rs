//! # Delete Confirmation Dialog
//! 
//! Diálogo de confirmação para exclusão de aplicações.

use eframe::egui::{self, RichText};

/// Resultado do diálogo de confirmação
pub enum DeleteConfirmResult {
    /// Nenhuma ação
    None,
    /// Confirmado a exclusão
    Confirmed(usize),
    /// Cancelado
    Cancelled,
}

/// Renderiza o diálogo de confirmação de exclusão.
/// 
/// # Argumentos
/// * `ctx` - Contexto do egui
/// * `app_name` - Nome da aplicação a ser excluída
/// * `index` - Índice da aplicação na lista
/// 
/// # Retorno
/// `DeleteConfirmResult` indicando a ação tomada
pub fn render_delete_confirm(
    ctx: &egui::Context,
    app_name: &str,
    index: usize,
) -> DeleteConfirmResult {
    let mut result = DeleteConfirmResult::None;
    
    egui::Window::new("⚠ Confirmar Exclusão")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(10.0);
            ui.label(format!("Tem certeza que deseja excluir \"{}\"?", app_name));
            ui.add_space(15.0);
            
            ui.horizontal(|ui| {
                if ui.button("Cancelar").clicked() {
                    result = DeleteConfirmResult::Cancelled;
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(
                        RichText::new("🗑 Excluir")
                            .color(egui::Color32::from_rgb(255, 100, 100))
                    ).clicked() {
                        result = DeleteConfirmResult::Confirmed(index);
                    }
                });
            });
        });
    
    result
}
