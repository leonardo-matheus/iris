//! # App Modal Dialog
//! 
//! Modal para adicionar/editar aplicações.

use eframe::egui::{self, RichText};
use crate::core::{AppConfig, IconInfo};
use crate::services::IconCache;

/// Estado do modal de aplicação
pub struct AppModalState {
    /// Aplicação sendo editada
    pub app: AppConfig,
    /// Novo comando sendo digitado
    pub new_command: String,
    /// Se o picker de ícones está aberto
    pub show_icon_picker: bool,
    /// Filtro de busca de ícones
    pub icon_search_filter: String,
    /// Índice da app sendo editada (None = nova app)
    pub edit_index: Option<usize>,
}

impl Default for AppModalState {
    fn default() -> Self {
        Self {
            app: AppConfig::default(),
            new_command: String::new(),
            show_icon_picker: false,
            icon_search_filter: String::new(),
            edit_index: None,
        }
    }
}

impl AppModalState {
    /// Cria um novo estado para adicionar uma aplicação
    pub fn new_app() -> Self {
        Self {
            app: AppConfig {
                id: crate::utils::uuid_simple(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Cria um estado para editar uma aplicação existente
    pub fn edit_app(app: AppConfig, index: usize) -> Self {
        Self {
            app,
            edit_index: Some(index),
            ..Default::default()
        }
    }
    
    /// Reseta o estado
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Resultado das ações do modal
pub enum AppModalResult {
    /// Modal fechado, nenhuma ação
    None,
    /// Salvar aplicação
    Save(AppConfig, Option<usize>),
    /// Cancelado
    Cancelled,
}

/// Renderiza o modal de adicionar/editar aplicação.
/// 
/// # Argumentos
/// * `ctx` - Contexto do egui
/// * `state` - Estado do modal
/// * `is_editing` - Se está editando ou adicionando
/// * `available_icons` - Lista de ícones disponíveis
/// * `icon_cache` - Cache de ícones
/// 
/// # Retorno
/// `AppModalResult` indicando a ação tomada
pub fn render_app_modal(
    ctx: &egui::Context,
    state: &mut AppModalState,
    is_editing: bool,
    available_icons: &[IconInfo],
    icon_cache: &mut IconCache,
) -> AppModalResult {
    let mut result = AppModalResult::None;
    
    let title = if is_editing {
        "✏ Editar Aplicação"
    } else {
        "➕ Nova Aplicação"
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(10.0);

            // Nome e Ícone
            render_name_and_icon(ui, state, icon_cache);
            
            // Icon picker
            if state.show_icon_picker {
                render_icon_picker(ui, state, available_icons, icon_cache);
            }

            ui.add_space(15.0);

            // Diretório de trabalho
            render_working_dir(ui, state);

            ui.add_space(15.0);

            // Lista de comandos
            render_commands_list(ui, state);

            ui.add_space(8.0);

            // Comandos sugeridos
            render_suggested_commands(ui, state);

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // Botões de ação
            result = render_modal_actions(ui, state, is_editing);
        });

    result
}

fn render_name_and_icon(ui: &mut egui::Ui, state: &mut AppModalState, icon_cache: &mut IconCache) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label("Ícone:");
            ui.horizontal(|ui| {
                let icon_size = egui::vec2(40.0, 40.0);
                
                if !state.app.icon_emoji.is_empty() {
                    if let Some(texture) = icon_cache.get_or_load(ui.ctx(), &state.app.icon_emoji) {
                        let btn = egui::ImageButton::new(egui::load::SizedTexture::new(
                            texture.id(),
                            icon_size,
                        ));
                        if ui.add(btn).on_hover_text("Clique para trocar").clicked() {
                            state.show_icon_picker = !state.show_icon_picker;
                        }
                    } else {
                        let btn = egui::Button::new(
                            RichText::new(&state.app.icon_emoji).size(10.0)
                        ).min_size(icon_size);
                        if ui.add(btn).on_hover_text("Clique para trocar").clicked() {
                            state.show_icon_picker = !state.show_icon_picker;
                        }
                    }
                } else {
                    let btn = egui::Button::new(
                        RichText::new("🚀").size(24.0)
                    ).min_size(icon_size);
                    if ui.add(btn).on_hover_text("Clique para escolher").clicked() {
                        state.show_icon_picker = !state.show_icon_picker;
                    }
                }
                
                if !state.app.icon_emoji.is_empty() {
                    ui.label(
                        RichText::new(&state.app.icon_emoji)
                            .size(11.0)
                            .color(egui::Color32::GRAY)
                    );
                }
            });
        });

        ui.add_space(10.0);

        ui.vertical(|ui| {
            ui.label("Nome da Aplicação:");
            ui.add(
                egui::TextEdit::singleline(&mut state.app.name)
                    .desired_width(300.0)
                    .hint_text("Minha Aplicação"),
            );
        });
    });
}

fn render_icon_picker(
    ui: &mut egui::Ui,
    state: &mut AppModalState,
    available_icons: &[IconInfo],
    icon_cache: &mut IconCache,
) {
    ui.add_space(10.0);
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.label("🎨 Escolha um ícone:");
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut state.icon_search_filter)
                    .desired_width(200.0)
                    .hint_text("🔍 Filtrar (ex: react, python, docker...)")
            );
            
            if ui.small_button("❌").clicked() {
                state.show_icon_picker = false;
                state.icon_search_filter.clear();
            }
        });
        
        ui.add_space(5.0);
        
        let filter = state.icon_search_filter.to_lowercase();
        let filtered_icons: Vec<&IconInfo> = available_icons
            .iter()
            .filter(|icon| filter.is_empty() || icon.name.to_lowercase().contains(&filter))
            .take(50)
            .collect();
        
        ui.label(
            RichText::new(format!("{} ícones encontrados", filtered_icons.len()))
                .size(11.0)
                .color(egui::Color32::GRAY)
        );
        
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for icon in filtered_icons {
                        let icon_name = icon.name.clone();
                        
                        if let Some(texture) = icon_cache.get_or_load(ui.ctx(), &icon_name) {
                            let btn = egui::ImageButton::new(egui::load::SizedTexture::new(
                                texture.id(),
                                egui::vec2(32.0, 32.0),
                            ));
                            if ui.add(btn).on_hover_text(&icon_name).clicked() {
                                state.app.icon_emoji = icon_name;
                                state.show_icon_picker = false;
                                state.icon_search_filter.clear();
                            }
                        } else {
                            let short_name: String = icon_name.chars().take(3).collect();
                            let btn = egui::Button::new(
                                RichText::new(&short_name).size(10.0)
                            ).min_size(egui::vec2(32.0, 32.0));
                            if ui.add(btn).on_hover_text(&icon_name).clicked() {
                                state.app.icon_emoji = icon_name;
                                state.show_icon_picker = false;
                                state.icon_search_filter.clear();
                            }
                        }
                    }
                });
            });
    });
}

fn render_working_dir(ui: &mut egui::Ui, state: &mut AppModalState) {
    ui.label("Pasta Inicial (Working Directory):");
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut state.app.working_dir)
                .desired_width(380.0)
                .hint_text("C:\\meus-projetos\\minha-app"),
        );
        if ui.button("📁 Selecionar").clicked() {
            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                state.app.working_dir = path.display().to_string();
            }
        }
    });
}

fn render_commands_list(ui: &mut egui::Ui, state: &mut AppModalState) {
    ui.label("Comandos (serão executados em sequência):");
    
    ui.add_space(5.0);

    let mut to_remove: Option<usize> = None;
    let mut to_move_up: Option<usize> = None;
    let mut to_move_down: Option<usize> = None;
    let commands_len = state.app.commands.len();

    egui::ScrollArea::vertical()
        .max_height(200.0)
        .show(ui, |ui| {
            for i in 0..commands_len {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    
                    ui.add_enabled_ui(i > 0, |ui| {
                        if ui.small_button("⬆").on_hover_text("Mover para cima").clicked() {
                            to_move_up = Some(i);
                        }
                    });
                    ui.add_enabled_ui(i < commands_len - 1, |ui| {
                        if ui.small_button("⬇").on_hover_text("Mover para baixo").clicked() {
                            to_move_down = Some(i);
                        }
                    });

                    ui.add(
                        egui::TextEdit::singleline(&mut state.app.commands[i])
                            .desired_width(320.0)
                            .font(egui::TextStyle::Monospace),
                    );

                    if ui.button("❌").on_hover_text("Remover comando").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
        });

    if let Some(i) = to_remove {
        state.app.commands.remove(i);
    }
    if let Some(i) = to_move_up {
        state.app.commands.swap(i, i - 1);
    }
    if let Some(i) = to_move_down {
        state.app.commands.swap(i, i + 1);
    }

    ui.add_space(10.0);

    // Adicionar novo comando
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut state.new_command)
                .desired_width(380.0)
                .hint_text("Digite um comando (ex: npm run dev)")
                .font(egui::TextStyle::Monospace),
        );

        let add_clicked = ui.button("➕ Adicionar").clicked();
        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

        if (add_clicked || enter_pressed) && !state.new_command.trim().is_empty() {
            state.app.commands.push(state.new_command.trim().to_string());
            state.new_command.clear();
        }
    });
}

fn render_suggested_commands(ui: &mut egui::Ui, state: &mut AppModalState) {
    ui.collapsing("💡 Comandos Comuns", |ui| {
        ui.horizontal_wrapped(|ui| {
            let suggestions = [
                "npm install",
                "npm run dev",
                "npm start",
                "yarn dev",
                "pnpm dev",
                "cargo run",
                "python main.py",
                "dotnet run",
                "go run .",
                "docker-compose up",
            ];
            for suggestion in suggestions {
                if ui.small_button(suggestion).clicked() {
                    state.app.commands.push(suggestion.to_string());
                }
            }
        });
    });
}

fn render_modal_actions(
    ui: &mut egui::Ui,
    state: &mut AppModalState,
    is_editing: bool,
) -> AppModalResult {
    let mut result = AppModalResult::None;
    
    ui.horizontal(|ui| {
        if ui.button(RichText::new("❌ Cancelar").size(14.0)).clicked() {
            result = AppModalResult::Cancelled;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let save_enabled = !state.app.name.trim().is_empty();
            
            ui.add_enabled_ui(save_enabled, |ui| {
                let save_text = if is_editing { "💾 Salvar" } else { "✅ Criar" };
                if ui.button(
                    RichText::new(save_text)
                        .size(14.0)
                        .color(egui::Color32::WHITE),
                ).clicked() {
                    let mut app = state.app.clone();
                    app.name = app.name.trim().to_string();
                    app.working_dir = app.working_dir.trim().to_string();
                    result = AppModalResult::Save(app, state.edit_index);
                }
            });

            if !save_enabled {
                ui.label(
                    RichText::new("⚠ Nome é obrigatório")
                        .size(12.0)
                        .color(egui::Color32::from_rgb(255, 200, 100)),
                );
            }
        });
    });
    
    result
}
