//! # App Hub - Aplicação Principal
//! 
//! Este módulo contém a estrutura principal da aplicação e a
//! implementação do trait `eframe::App`.

use std::sync::Arc;
use std::time::Duration;
use eframe::egui;

use crate::core::{AppConfig, AppState, ConfigManager, IconInfo};
use crate::services::{IconCache, ProcessManager, load_available_icons};
use crate::ui::components::{render_app_card, render_header, render_footer, render_empty_state, render_no_results};
use crate::ui::dialogs::{AppModalState, AppModalResult, DeleteConfirmResult, render_app_modal, render_delete_confirm};
use crate::ui::theme;
use crate::utils::uuid_simple;

/// Aplicação principal do Hub Iris.
/// 
/// Gerencia o estado da UI e coordena todos os microserviços.
pub struct AppHub {
    // Dados
    state: AppState,
    config_manager: ConfigManager,
    
    // Serviços
    process_manager: ProcessManager,
    icon_cache: IconCache,
    available_icons: Vec<IconInfo>,
    
    // Estado da UI
    search_filter: String,
    show_add_modal: bool,
    show_edit_modal: bool,
    modal_state: AppModalState,
    show_delete_confirm: Option<usize>,
}

impl AppHub {
    /// Cria uma nova instância da aplicação
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_manager = ConfigManager::new();
        let mut state = config_manager.load();
        
        // Garantir que todos os apps tenham ID
        for app in &mut state.apps {
            if app.id.is_empty() {
                app.id = uuid_simple();
            }
        }

        Self {
            state,
            config_manager,
            process_manager: ProcessManager::new(),
            icon_cache: IconCache::new(),
            available_icons: load_available_icons(),
            search_filter: String::new(),
            show_add_modal: false,
            show_edit_modal: false,
            modal_state: AppModalState::default(),
            show_delete_confirm: None,
        }
    }

    /// Salva o estado atual em disco
    fn save_state(&self) {
        if let Err(e) = self.config_manager.save(&self.state) {
            eprintln!("Erro ao salvar: {}", e);
        }
    }

    /// Inicia o modal para adicionar nova aplicação
    fn start_add_app(&mut self) {
        self.modal_state = AppModalState::new_app();
        self.show_add_modal = true;
    }

    /// Inicia o modal para editar uma aplicação
    fn start_edit_app(&mut self, index: usize) {
        if let Some(app) = self.state.apps.get(index) {
            self.modal_state = AppModalState::edit_app(app.clone(), index);
            self.show_edit_modal = true;
        }
    }

    /// Fecha o modal e limpa o estado
    fn close_modal(&mut self) {
        self.show_add_modal = false;
        self.show_edit_modal = false;
        self.modal_state.reset();
    }

    /// Processa o resultado do modal de aplicação
    fn handle_modal_result(&mut self, result: AppModalResult) {
        match result {
            AppModalResult::Save(app, edit_index) => {
                if let Some(index) = edit_index {
                    self.state.apps[index] = app;
                } else {
                    self.state.add_app(app);
                }
                self.save_state();
                self.close_modal();
            }
            AppModalResult::Cancelled => {
                self.close_modal();
            }
            AppModalResult::None => {}
        }
    }

    /// Processa o resultado do diálogo de confirmação de exclusão
    fn handle_delete_result(&mut self, result: DeleteConfirmResult) {
        match result {
            DeleteConfirmResult::Confirmed(index) => {
                self.state.remove_app(index);
                self.save_state();
                self.show_delete_confirm = None;
            }
            DeleteConfirmResult::Cancelled => {
                self.show_delete_confirm = None;
            }
            DeleteConfirmResult::None => {}
        }
    }

    /// Exporta as configurações para um arquivo
    fn export_config(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Exportar configurações do Iris")
            .set_file_name("iris-config.json")
            .add_filter("JSON", &["json"])
            .save_file()
        {
            if let Err(e) = self.config_manager.export(&self.state, &path) {
                eprintln!("{}", e);
            }
        }
    }

    /// Importa configurações de um arquivo
    fn import_config(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Importar configurações do Iris")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            match self.config_manager.import(&path) {
                Ok(imported_state) => {
                    self.state.apps.extend(imported_state.apps);
                    self.save_state();
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
    }

    /// Renderiza a área central com os cards das aplicações
    fn render_central_panel(&mut self, ui: &mut egui::Ui) {
        if self.state.apps.is_empty() {
            if render_empty_state(ui) {
                self.start_add_app();
            }
        } else {
            self.render_apps_grid(ui);
        }
    }

    /// Renderiza o grid de aplicações
    fn render_apps_grid(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(20.0);

            // Filtrar apps pela busca
            let filtered_indices: Vec<usize> = self.state.apps
                .iter()
                .enumerate()
                .filter(|(_, app)| {
                    self.search_filter.is_empty()
                        || app.name.to_lowercase().contains(&self.search_filter.to_lowercase())
                        || app.working_dir.to_lowercase().contains(&self.search_filter.to_lowercase())
                })
                .map(|(i, _)| i)
                .collect();

            if filtered_indices.is_empty() {
                render_no_results(ui, &self.search_filter);
            } else {
                self.render_filtered_apps(ui, &filtered_indices);
            }

            ui.add_space(20.0);
        });
    }

    /// Renderiza as aplicações filtradas em um grid
    fn render_filtered_apps(&mut self, ui: &mut egui::Ui, filtered_indices: &[usize]) {
        let available_width = ui.available_width();
        let card_width = 260.0;
        let spacing = 16.0;
        let cards_per_row = ((available_width + spacing) / (card_width + spacing)).floor() as usize;
        let cards_per_row = cards_per_row.max(1);

        let mut app_to_launch: Option<usize> = None;
        let mut app_to_stop: Option<usize> = None;
        let mut app_to_restart: Option<usize> = None;
        let mut app_to_edit: Option<usize> = None;
        let mut app_to_delete: Option<usize> = None;

        egui::Grid::new("apps_grid")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                for (col, &index) in filtered_indices.iter().enumerate() {
                    let app = &self.state.apps[index];
                    let is_running = self.process_manager.is_running(&app.id);
                    let is_loading = self.process_manager.is_loading(&app.id);
                    
                    let actions = render_app_card(
                        ui,
                        app,
                        is_running,
                        is_loading,
                        &mut self.icon_cache,
                    );
                    
                    if actions.start_clicked {
                        app_to_launch = Some(index);
                    }
                    if actions.stop_clicked {
                        app_to_stop = Some(index);
                    }
                    if actions.restart_clicked {
                        app_to_restart = Some(index);
                    }
                    if actions.edit_clicked {
                        app_to_edit = Some(index);
                    }
                    if actions.delete_clicked {
                        app_to_delete = Some(index);
                    }

                    if (col + 1) % cards_per_row == 0 {
                        ui.end_row();
                    }
                }
            });

        // Executar ações
        if let Some(index) = app_to_launch {
            let app = self.state.apps[index].clone();
            self.process_manager.launch_app(&app);
        }
        if let Some(index) = app_to_stop {
            let app = &self.state.apps[index];
            self.process_manager.stop_app(&app.id, Some(&app.name), Some(&app.commands));
        }
        if let Some(index) = app_to_restart {
            let app = self.state.apps[index].clone();
            self.process_manager.restart_app(&app);
        }
        if let Some(index) = app_to_edit {
            self.start_edit_app(index);
        }
        if let Some(index) = app_to_delete {
            self.show_delete_confirm = Some(index);
        }
    }
}

impl eframe::App for AppHub {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Aplicar tema
        theme::apply_theme(ctx);
        
        // Limpar processos mortos
        self.process_manager.cleanup_dead_processes();
        
        // Configurar repaint
        let needs_fast_repaint = self.process_manager.has_loading() || self.process_manager.has_running();
        if needs_fast_repaint {
            ctx.request_repaint_after(Duration::from_millis(250));
        } else {
            ctx.request_repaint_after(Duration::from_secs(2));
        }

        // Header
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(22, 22, 26))
                .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            )
            .show(ctx, |ui| {
                let header_actions = render_header(ui, &mut self.search_filter);
                
                if header_actions.add_app_clicked {
                    self.start_add_app();
                }
                if header_actions.export_clicked {
                    self.export_config();
                }
                if header_actions.import_clicked {
                    self.import_config();
                }
            });

        // Footer
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(22, 22, 26))
                .inner_margin(egui::Margin::symmetric(20.0, 12.0))
            )
            .show(ctx, |ui| {
                render_footer(ui, self.state.app_count(), self.process_manager.running_count());
            });

        // Área central
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(18, 18, 22))
                .inner_margin(egui::Margin::same(24.0))
            )
            .show(ctx, |ui| {
                self.render_central_panel(ui);
            });

        // Modais
        if self.show_add_modal || self.show_edit_modal {
            let result = render_app_modal(
                ctx,
                &mut self.modal_state,
                self.show_edit_modal,
                &self.available_icons,
                &mut self.icon_cache,
            );
            self.handle_modal_result(result);
        }

        // Diálogo de confirmação de exclusão
        if let Some(index) = self.show_delete_confirm {
            let app_name = self.state.apps.get(index)
                .map(|a| a.name.clone())
                .unwrap_or_default();
            let result = render_delete_confirm(ctx, &app_name, index);
            self.handle_delete_result(result);
        }
    }
}
