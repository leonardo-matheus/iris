#![windows_subsystem = "windows"]

use eframe::egui::{self, ColorImage, TextureHandle};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::sync::{Arc, Mutex};

/// Estrutura que representa uma aplicação configurada
#[derive(Clone, Serialize, Deserialize, Default)]
struct AppConfig {
    id: String,
    name: String,
    icon_emoji: String,
    working_dir: String,
    commands: Vec<String>,
}

/// Gera um ID simples baseado em timestamp
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}{}", duration.as_secs(), duration.subsec_nanos())
}

/// Informações do processo em execução
struct RunningProcess {
    #[allow(dead_code)]
    child: Child,
    console_pid: Option<u32>,
    started_at: std::time::Instant,
}

/// Estado da aplicação principal
#[derive(Serialize, Deserialize)]
struct AppState {
    apps: Vec<AppConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { apps: Vec::new() }
    }
}

/// Estrutura para armazenar informações de ícone
#[derive(Clone)]
struct IconInfo {
    name: String,
    filename: String,
}

/// Cache de texturas dos ícones
struct IconCache {
    textures: HashMap<String, TextureHandle>,
}

impl IconCache {
    fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
    
    fn get_or_load(&mut self, ctx: &egui::Context, icon_name: &str) -> Option<TextureHandle> {
        if let Some(texture) = self.textures.get(icon_name) {
            return Some(texture.clone());
        }
        
        // Tenta carregar o SVG
        let svg_path = format!("assets/langs/{}-original.svg", icon_name);
        if let Ok(svg_data) = fs::read_to_string(&svg_path) {
            if let Some(image) = render_svg_to_image(&svg_data, 32, 32) {
                let texture = ctx.load_texture(
                    icon_name,
                    image,
                    egui::TextureOptions::LINEAR,
                );
                self.textures.insert(icon_name.to_string(), texture.clone());
                return Some(texture);
            }
        }
        
        None
    }
}

/// Renderiza SVG para ColorImage
fn render_svg_to_image(svg_data: &str, width: u32, height: u32) -> Option<ColorImage> {
    let opts = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_str(svg_data, &opts).ok()?;
    
    let size = tree.size();
    let scale_x = width as f32 / size.width();
    let scale_y = height as f32 / size.height();
    let scale = scale_x.min(scale_y);
    
    let scaled_width = (size.width() * scale) as u32;
    let scaled_height = (size.height() * scale) as u32;
    
    let mut pixmap = resvg::tiny_skia::Pixmap::new(scaled_width, scaled_height)?;
    
    let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    
    let pixels: Vec<egui::Color32> = pixmap
        .pixels()
        .iter()
        .map(|p| egui::Color32::from_rgba_premultiplied(p.red(), p.green(), p.blue(), p.alpha()))
        .collect();
    
    Some(ColorImage {
        size: [scaled_width as usize, scaled_height as usize],
        pixels,
    })
}

/// Carrega a lista de ícones disponíveis
fn load_available_icons() -> Vec<IconInfo> {
    let mut icons = Vec::new();
    
    if let Ok(entries) = fs::read_dir("assets/langs") {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            if filename.ends_with("-original.svg") {
                let name = filename
                    .strip_suffix("-original.svg")
                    .unwrap_or(&filename)
                    .to_string();
                icons.push(IconInfo {
                    name: name.clone(),
                    filename,
                });
            }
        }
    }
    
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    icons
}

/// Aplicação principal do Hub
struct AppHub {
    state: AppState,
    show_add_modal: bool,
    show_edit_modal: bool,
    show_icon_picker: bool,
    edit_index: Option<usize>,
    modal_app: AppConfig,
    new_command: String,
    config_path: PathBuf,
    search_filter: String,
    icon_search_filter: String,
    show_delete_confirm: Option<usize>,
    running_apps: Arc<Mutex<HashMap<String, RunningProcess>>>,
    loading_apps: Arc<Mutex<HashSet<String>>>,
    available_icons: Vec<IconInfo>,
    icon_cache: IconCache,
}

impl AppHub {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_path = Self::get_config_path();
        let mut state = Self::load_state(&config_path);
        
        // Garantir que todos os apps tenham ID
        for app in &mut state.apps {
            if app.id.is_empty() {
                app.id = uuid_simple();
            }
        }

        Self {
            state,
            show_add_modal: false,
            show_edit_modal: false,
            show_icon_picker: false,
            edit_index: None,
            modal_app: AppConfig::default(),
            new_command: String::new(),
            config_path,
            search_filter: String::new(),
            show_delete_confirm: None,
            running_apps: Arc::new(Mutex::new(HashMap::new())),
            loading_apps: Arc::new(Mutex::new(HashSet::new())),
            available_icons: load_available_icons(),
            icon_cache: IconCache::new(),
            icon_search_filter: String::new(),
        }
    }

    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("iris");
        fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }

    fn load_state(path: &PathBuf) -> AppState {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        AppState::default()
    }

    fn save_state(&self) {
        if let Ok(json) = serde_json::to_string_pretty(&self.state) {
            fs::write(&self.config_path, json).ok();
        }
    }

    fn launch_app(&mut self, app: &AppConfig) {
        if app.commands.is_empty() {
            return;
        }

        // Para o processo anterior se existir
        self.stop_app(&app.id);
        
        // Marca como loading
        {
            let mut loading = self.loading_apps.lock().unwrap();
            loading.insert(app.id.clone());
        }

        // Clona os dados necessários para a thread
        let app_clone = app.clone();
        let running_apps = Arc::clone(&self.running_apps);
        let loading_apps = Arc::clone(&self.loading_apps);

        // Executa em uma thread separada para não bloquear a UI
        std::thread::spawn(move || {
            // Cria um arquivo batch temporário
            let temp_dir = std::env::temp_dir();
            let batch_file = temp_dir.join(format!("iris_{}.bat", app_clone.id));
            
            // Monta o conteúdo do batch
            let mut batch_content = String::new();
            batch_content.push_str("@echo off\n");
            batch_content.push_str(&format!("title [IRIS] {}\n", app_clone.name));
            
            if !app_clone.working_dir.is_empty() {
                batch_content.push_str(&format!("cd /d \"{}\"\n", app_clone.working_dir));
            }
            
            let commands = &app_clone.commands;
            let mut i = 0;
            while i < commands.len() {
                let cmd = &commands[i];
                let cmd_lower = cmd.to_lowercase();
                
                // Verifica se o próximo comando parece ser um input (só números/versão ou S/N)
                let next_is_input = if i + 1 < commands.len() {
                    let next = &commands[i + 1];
                    // É input se for: apenas números com pontos (versão), ou S/N, ou texto curto sem espaços
                    next.chars().all(|c| c.is_numeric() || c == '.')
                        || next.eq_ignore_ascii_case("s")
                        || next.eq_ignore_ascii_case("n")
                        || next.eq_ignore_ascii_case("y")
                } else {
                    false
                };
                
                // Se o comando atual é um .bat/.cmd e o próximo é input, usa redirecionamento de arquivo
                if next_is_input && (cmd_lower.ends_with(".bat") || cmd_lower.ends_with(".cmd")) {
                    let input = &commands[i + 1];
                    // Cria um arquivo temporário com o input e redireciona para o script
                    let input_file = format!("iris_input_{}.txt", app_clone.id);
                    batch_content.push_str(&format!("echo {}> %TEMP%\\{}\n", input, input_file));
                    batch_content.push_str(&format!("call {} < %TEMP%\\{}\n", cmd, input_file));
                    batch_content.push_str(&format!("del %TEMP%\\{} 2>nul\n", input_file));
                    i += 2; // Pula o próximo comando pois já foi usado como input
                    // Restaura o título após o comando
                    batch_content.push_str(&format!("title [IRIS] {}\n", app_clone.name));
                    continue;
                }
                
                let needs_call = cmd_lower.starts_with("npm ")
                    || cmd_lower.starts_with("yarn ")
                    || cmd_lower.starts_with("pnpm ")
                    || cmd_lower.starts_with("npx ")
                    || cmd_lower.starts_with("dotnet ")
                    || cmd_lower.starts_with("cargo ")
                    || cmd_lower.ends_with(".bat")
                    || cmd_lower.ends_with(".cmd");
                
                if needs_call {
                    batch_content.push_str(&format!("call {}\n", cmd));
                } else {
                    batch_content.push_str(&format!("{}\n", cmd));
                }
                
                // Restaura o título após cada comando para facilitar o taskkill
                batch_content.push_str(&format!("title [IRIS] {}\n", app_clone.name));
                
                i += 1;
            }
            
            // Garante que o título final está correto
            batch_content.push_str(&format!("title [IRIS] {}\n", app_clone.name));
            batch_content.push_str("cmd /k\n");
            
            if fs::write(&batch_file, &batch_content).is_err() {
                let mut loading = loading_apps.lock().unwrap();
                loading.remove(&app_clone.id);
                return;
            }

            // Executa o batch
            let child = Command::new("cmd")
                .args(["/C", "start", "", &batch_file.to_string_lossy()])
                .spawn();

            // Aguarda um pouco para o console abrir
            std::thread::sleep(std::time::Duration::from_millis(800));

            if let Ok(child) = child {
                // Tenta obter o PID do processo do console
                let console_pid = Self::find_console_pid_static(&app_clone.name);
                
                let mut running = running_apps.lock().unwrap();
                running.insert(
                    app_clone.id.clone(),
                    RunningProcess {
                        child,
                        console_pid,
                        started_at: std::time::Instant::now(),
                    },
                );
            }
            
            // Remove do loading
            let mut loading = loading_apps.lock().unwrap();
            loading.remove(&app_clone.id);
        });
    }
    
    fn find_console_pid_static(title: &str) -> Option<u32> {
        let search_title = format!("[IRIS] {}", title);
        
        // Tenta várias vezes para garantir que o console já abriu
        for _ in 0..5 {
            // Busca pelo título exato com prefixo
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Get-Process cmd -ErrorAction SilentlyContinue | Where-Object {{$_.MainWindowTitle -eq '{}'}} | Select-Object -First 1 -ExpandProperty Id",
                        search_title
                    ),
                ])
                .creation_flags(0x08000000)
                .output()
                .ok()?;

            let pid_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(pid) = pid_str.trim().parse() {
                return Some(pid);
            }
            
            // Tenta com -like se -eq não funcionou
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Get-Process cmd -ErrorAction SilentlyContinue | Where-Object {{$_.MainWindowTitle -like '*[IRIS]*{}*'}} | Select-Object -First 1 -ExpandProperty Id",
                        title
                    ),
                ])
                .creation_flags(0x08000000)
                .output()
                .ok()?;

            let pid_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(pid) = pid_str.trim().parse() {
                return Some(pid);
            }
            
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        
        None
    }

    fn stop_app(&mut self, app_id: &str) {
        // Pega as informações do app
        let app_info = self.state.apps.iter()
            .find(|a| a.id == app_id)
            .map(|a| (a.name.clone(), a.commands.clone()));
            
        let mut running = self.running_apps.lock().unwrap();
        
        if let Some(mut process) = running.remove(app_id) {
            // Estratégia 1: Mata pelo título do comando npm/yarn/etc que está rodando
            // O título da janela muda para o comando (ex: "npm run preview --host")
            if let Some((_, ref commands)) = app_info {
                for cmd in commands {
                    // Tenta matar pelo título que é o próprio comando
                    let _ = Command::new("taskkill")
                        .args(["/F", "/FI", &format!("WINDOWTITLE eq {}", cmd)])
                        .creation_flags(0x08000000)
                        .output();
                    
                    // Tenta com wildcard
                    let _ = Command::new("taskkill")
                        .args(["/F", "/FI", &format!("WINDOWTITLE eq {}*", cmd)])
                        .creation_flags(0x08000000)
                        .output();
                }
            }

            // Estratégia 2: Tenta pelo título [IRIS] Nome
            if let Some((ref name, _)) = app_info {
                let _ = Command::new("taskkill")
                    .args(["/F", "/FI", &format!("WINDOWTITLE eq [IRIS] {}", name)])
                    .creation_flags(0x08000000)
                    .output();
            }

            // Estratégia 3: Se temos o PID, mata a árvore inteira
            if let Some(pid) = process.console_pid {
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output();
            }

            // Estratégia 4: Usa WMIC para matar pelo CommandLine do batch
            let batch_name = format!("iris_{}.bat", app_id);
            let _ = Command::new("cmd")
                .args(["/C", &format!("wmic process where \"CommandLine like '%{}%'\" call terminate 2>nul", batch_name)])
                .creation_flags(0x08000000)
                .output();

            // Estratégia 5: Mata processos cmd.exe que têm npm/node/vite no título
            for pattern in ["npm*", "node*", "vite*", "yarn*", "pnpm*"] {
                let _ = Command::new("taskkill")
                    .args(["/F", "/FI", &format!("WINDOWTITLE eq {}", pattern)])
                    .creation_flags(0x08000000)
                    .output();
            }
            
            // Mata o processo child diretamente
            let _ = process.child.kill();
        }
    }

    fn restart_app(&mut self, app: &AppConfig) {
        self.stop_app(&app.id);
        // Pequeno delay antes de reiniciar
        std::thread::sleep(std::time::Duration::from_millis(200));
        self.launch_app(app);
    }

    fn is_app_running(&self, app_id: &str) -> bool {
        let running = self.running_apps.lock().unwrap();
        running.contains_key(app_id)
    }
    
    fn is_app_loading(&self, app_id: &str) -> bool {
        let loading = self.loading_apps.lock().unwrap();
        loading.contains(app_id)
    }

    fn cleanup_dead_processes(&mut self) {
        let mut running = self.running_apps.lock().unwrap();
        let mut to_remove = Vec::new();
        
        for (app_id, process) in running.iter() {
            // Só verifica se tem PID
            if let Some(pid) = process.console_pid {
                // Verifica se o processo ainda existe
                let output = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                    .output();
                
                if let Ok(output) = output {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Se não contém "cmd.exe", o processo morreu
                    if !output_str.to_lowercase().contains("cmd.exe") {
                        to_remove.push(app_id.clone());
                    }
                }
            }
            // Se não tem PID, não remove automaticamente - o usuário precisa usar Stop
        }
        
        for app_id in to_remove {
            running.remove(&app_id);
        }
    }

    fn render_app_card(&mut self, ui: &mut egui::Ui, index: usize) -> (bool, bool, bool) {
        let app = &self.state.apps[index];
        let app_id = app.id.clone();
        let is_running = self.is_app_running(&app_id);
        let is_loading = self.is_app_loading(&app_id);
        
        let mut start_clicked = false;
        let mut stop_clicked = false;
        let mut restart_clicked = false;
        let mut edit_clicked = false;
        let mut delete_clicked = false;

        // Cores modernas baseadas no status
        let (bg_color, border_color, glow_color) = if is_running {
            (
                egui::Color32::from_rgb(25, 35, 30),
                egui::Color32::from_rgb(34, 197, 94),
                egui::Color32::from_rgba_unmultiplied(34, 197, 94, 30),
            )
        } else if is_loading {
            (
                egui::Color32::from_rgb(25, 30, 40),
                egui::Color32::from_rgb(99, 102, 241),
                egui::Color32::from_rgba_unmultiplied(99, 102, 241, 30),
            )
        } else {
            (
                egui::Color32::from_rgb(32, 32, 38),
                egui::Color32::from_rgb(55, 55, 62),
                egui::Color32::TRANSPARENT,
            )
        };

        // Card com tamanho fixo
        let card_width = 250.0;
        let card_height = 240.0;
        
        let response = egui::Frame::none()
            .fill(bg_color)
            .rounding(16.0)
            .inner_margin(egui::Margin::same(18.0))
            .stroke(egui::Stroke::new(1.5, border_color))
            .shadow(egui::epaint::Shadow {
                offset: egui::vec2(0.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: glow_color,
            })
            .show(ui, |ui| {
                ui.set_width(card_width);
                ui.set_min_height(card_height);

                ui.vertical(|ui| {
                    ui.set_min_height(card_height);
                    
                    // Header do card
                    ui.horizontal(|ui| {
                        // Ícone da aplicação com container
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(45, 45, 52))
                            .rounding(10.0)
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.set_min_size(egui::vec2(44.0, 44.0));
                                if !app.icon_emoji.is_empty() {
                                    if let Some(texture) = self.icon_cache.get_or_load(ui.ctx(), &app.icon_emoji) {
                                        ui.image(egui::load::SizedTexture::new(
                                            texture.id(),
                                            egui::vec2(28.0, 28.0),
                                        ));
                                    } else {
                                        ui.label(egui::RichText::new("🚀").size(24.0));
                                    }
                                } else {
                                    ui.label(egui::RichText::new("🚀").size(24.0));
                                }
                            });
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.horizontal(|ui| {
                                ui.style_mut().spacing.item_spacing = egui::vec2(2.0, 0.0);
                                
                                if !is_running && !is_loading {
                                    let del_btn = egui::Button::new(
                                        egui::RichText::new("x").size(14.0).color(egui::Color32::from_rgb(180, 80, 80))
                                    )
                                    .fill(egui::Color32::from_rgb(60, 40, 40))
                                    .rounding(6.0)
                                    .min_size(egui::vec2(24.0, 24.0));
                                    if ui.add(del_btn).on_hover_text("Deletar").clicked() {
                                        delete_clicked = true;
                                    }
                                }
                                if !is_loading {
                                    let edit_btn = egui::Button::new(
                                        egui::RichText::new("...").size(12.0).color(egui::Color32::from_rgb(160, 160, 170))
                                    )
                                    .fill(egui::Color32::from_rgb(50, 50, 58))
                                    .rounding(6.0)
                                    .min_size(egui::vec2(24.0, 24.0));
                                    if ui.add(edit_btn).on_hover_text("Editar").clicked() {
                                        edit_clicked = true;
                                    }
                                }
                            });
                        });
                    });

                    ui.add_space(12.0);

                    // Nome da aplicação
                    ui.label(
                        egui::RichText::new(&app.name)
                            .size(17.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );

                    ui.add_space(4.0);

                    // Status badge - sempre reserva espaço para manter alinhamento
                    ui.allocate_ui_with_layout(
                        egui::vec2(card_width, 18.0),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            if is_running {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(34, 55, 40))
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new("▶ Executando")
                                                .size(10.0)
                                                .color(egui::Color32::from_rgb(34, 197, 94)),
                                        );
                                    });
                            } else if is_loading {
                                egui::Frame::none()
                                    .fill(egui::Color32::from_rgb(40, 40, 60))
                                    .rounding(4.0)
                                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new("⏳ Iniciando...")
                                                .size(10.0)
                                                .color(egui::Color32::from_rgb(99, 102, 241)),
                                        );
                                    });
                            }
                        },
                    );

                    ui.add_space(8.0);

                    // Info do projeto - sempre reserva espaço para o working_dir
                    ui.allocate_ui_with_layout(
                        egui::vec2(card_width, 16.0),
                        egui::Layout::left_to_right(egui::Align::Min),
                        |ui| {
                            if !app.working_dir.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("📂 {}", truncate_path(&app.working_dir, 28)))
                                        .size(11.0)
                                        .color(egui::Color32::from_rgb(120, 120, 130)),
                                );
                            }
                        },
                    );

                    ui.label(
                        egui::RichText::new(format!("⚡ {} comando(s)", app.commands.len()))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 100, 110)),
                    );

                    // Preencher espaço restante antes dos botões
                    ui.add_space(ui.available_height() - 46.0);

                    // Botões de ação com largura fixa - sempre no final do card
                    let button_width = card_width - 36.0;
                    
                    if is_loading {
                        let button = egui::Button::new(
                            egui::RichText::new("⏳ Iniciando...")
                                .size(13.0)
                                .color(egui::Color32::from_rgb(150, 150, 160)),
                        )
                        .fill(egui::Color32::from_rgb(50, 50, 58))
                        .rounding(10.0)
                        .min_size(egui::vec2(button_width, 36.0));
                        ui.add_enabled(false, button);
                    } else if is_running {
                        ui.horizontal(|ui| {
                            let btn_width = (button_width - 8.0) / 2.0;
                            
                            let stop_button = egui::Button::new(
                                egui::RichText::new("■ Stop")
                                    .size(12.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(239, 68, 68))
                            .rounding(10.0)
                            .min_size(egui::vec2(btn_width, 36.0));

                            if ui.add(stop_button).clicked() {
                                stop_clicked = true;
                            }

                            let restart_button = egui::Button::new(
                                egui::RichText::new("↻ Restart")
                                    .size(12.0)
                                    .color(egui::Color32::WHITE),
                            )
                            .fill(egui::Color32::from_rgb(245, 158, 11))
                            .rounding(10.0)
                            .min_size(egui::vec2(btn_width, 36.0));

                            if ui.add(restart_button).clicked() {
                                restart_clicked = true;
                            }
                        });
                    } else {
                        let button = egui::Button::new(
                            egui::RichText::new("▶  Executar")
                                .size(13.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(99, 102, 241))
                        .rounding(10.0)
                        .min_size(egui::vec2(button_width, 36.0));

                        if ui.add(button).clicked() {
                            start_clicked = true;
                        }
                    }
                });
            });

        // Hover effect
        if response.response.hovered() {
            ui.ctx().request_repaint();
        }

        if edit_clicked {
            self.modal_app = app.clone();
            self.edit_index = Some(index);
            self.show_edit_modal = true;
        }

        if delete_clicked {
            self.show_delete_confirm = Some(index);
        }

        (start_clicked, stop_clicked, restart_clicked)
    }

    fn render_modal(&mut self, ctx: &egui::Context) {
        let is_editing = self.show_edit_modal;
        let title = if is_editing {
            "✏ Editar Aplicação"
        } else {
            "➕ Nova Aplicação"
        };

        let should_show = self.show_add_modal || self.show_edit_modal;

        if should_show {
            egui::Window::new(title)
                .collapsible(false)
                .resizable(true)
                .default_width(500.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add_space(10.0);

                    // Nome e Ícone lado a lado
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Ícone:");
                            ui.horizontal(|ui| {
                                // Mostra o ícone atual ou placeholder
                                let icon_size = egui::vec2(40.0, 40.0);
                                
                                if !self.modal_app.icon_emoji.is_empty() {
                                    if let Some(texture) = self.icon_cache.get_or_load(ui.ctx(), &self.modal_app.icon_emoji) {
                                        let btn = egui::ImageButton::new(egui::load::SizedTexture::new(
                                            texture.id(),
                                            icon_size,
                                        ));
                                        if ui.add(btn).on_hover_text("Clique para trocar").clicked() {
                                            self.show_icon_picker = !self.show_icon_picker;
                                        }
                                    } else {
                                        // Fallback: mostra o nome
                                        let btn = egui::Button::new(
                                            egui::RichText::new(&self.modal_app.icon_emoji).size(10.0)
                                        ).min_size(icon_size);
                                        if ui.add(btn).on_hover_text("Clique para trocar").clicked() {
                                            self.show_icon_picker = !self.show_icon_picker;
                                        }
                                    }
                                } else {
                                    let btn = egui::Button::new(
                                        egui::RichText::new("🚀").size(24.0)
                                    ).min_size(icon_size);
                                    if ui.add(btn).on_hover_text("Clique para escolher").clicked() {
                                        self.show_icon_picker = !self.show_icon_picker;
                                    }
                                }
                                
                                // Mostra o nome do ícone selecionado
                                if !self.modal_app.icon_emoji.is_empty() {
                                    ui.label(
                                        egui::RichText::new(&self.modal_app.icon_emoji)
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
                                egui::TextEdit::singleline(&mut self.modal_app.name)
                                    .desired_width(300.0)
                                    .hint_text("Minha Aplicação"),
                            );
                        });
                    });
                    
                    // Popup de seleção de ícone
                    if self.show_icon_picker {
                        ui.add_space(10.0);
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("🎨 Escolha um ícone:");
                                ui.add_space(10.0);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.icon_search_filter)
                                        .desired_width(200.0)
                                        .hint_text("🔍 Filtrar (ex: react, python, docker...)")
                                );
                                
                                if ui.small_button("❌").clicked() {
                                    self.show_icon_picker = false;
                                    self.icon_search_filter.clear();
                                }
                            });
                            
                            ui.add_space(5.0);
                            
                            // Filtra os ícones
                            let filter = self.icon_search_filter.to_lowercase();
                            let filtered_icons: Vec<&IconInfo> = self.available_icons
                                .iter()
                                .filter(|icon| filter.is_empty() || icon.name.to_lowercase().contains(&filter))
                                .take(50) // Limita para não sobrecarregar
                                .collect();
                            
                            ui.label(
                                egui::RichText::new(format!("{} ícones encontrados", filtered_icons.len()))
                                    .size(11.0)
                                    .color(egui::Color32::GRAY)
                            );
                            
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        for icon in filtered_icons {
                                            let icon_name = icon.name.clone();
                                            
                                            // Tenta renderizar o ícone
                                            if let Some(texture) = self.icon_cache.get_or_load(ui.ctx(), &icon_name) {
                                                let btn = egui::ImageButton::new(egui::load::SizedTexture::new(
                                                    texture.id(),
                                                    egui::vec2(32.0, 32.0),
                                                ));
                                                if ui.add(btn).on_hover_text(&icon_name).clicked() {
                                                    self.modal_app.icon_emoji = icon_name;
                                                    self.show_icon_picker = false;
                                                    self.icon_search_filter.clear();
                                                }
                                            } else {
                                                // Fallback: botão com texto
                                                let short_name: String = icon_name.chars().take(3).collect();
                                                let btn = egui::Button::new(
                                                    egui::RichText::new(&short_name).size(10.0)
                                                ).min_size(egui::vec2(32.0, 32.0));
                                                if ui.add(btn).on_hover_text(&icon_name).clicked() {
                                                    self.modal_app.icon_emoji = icon_name;
                                                    self.show_icon_picker = false;
                                                    self.icon_search_filter.clear();
                                                }
                                            }
                                        }
                                    });
                                });
                        });
                    }

                    ui.add_space(15.0);

                    // Diretório de trabalho
                    ui.label("Pasta Inicial (Working Directory):");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.modal_app.working_dir)
                                .desired_width(380.0)
                                .hint_text("C:\\meus-projetos\\minha-app"),
                        );
                        if ui.button("📁 Selecionar").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.modal_app.working_dir = path.display().to_string();
                            }
                        }
                    });

                    ui.add_space(15.0);

                    // Lista de comandos
                    ui.label("Comandos (serão executados em sequência):");
                    
                    ui.add_space(5.0);

                    // Área scrollable para os comandos
                    let mut to_remove: Option<usize> = None;
                    let mut to_move_up: Option<usize> = None;
                    let mut to_move_down: Option<usize> = None;
                    let commands_len = self.modal_app.commands.len();

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for i in 0..commands_len {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}.", i + 1));
                                    
                                    // Botões de reordenar
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
                                        egui::TextEdit::singleline(&mut self.modal_app.commands[i])
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
                        self.modal_app.commands.remove(i);
                    }
                    if let Some(i) = to_move_up {
                        self.modal_app.commands.swap(i, i - 1);
                    }
                    if let Some(i) = to_move_down {
                        self.modal_app.commands.swap(i, i + 1);
                    }

                    ui.add_space(10.0);

                    // Adicionar novo comando
                    ui.horizontal(|ui| {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.new_command)
                                .desired_width(380.0)
                                .hint_text("Digite um comando (ex: npm run dev)")
                                .font(egui::TextStyle::Monospace),
                        );

                        let add_clicked = ui.button("➕ Adicionar").clicked();
                        let enter_pressed = response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

                        if (add_clicked || enter_pressed) && !self.new_command.trim().is_empty() {
                            self.modal_app.commands.push(self.new_command.trim().to_string());
                            self.new_command.clear();
                        }
                    });

                    ui.add_space(8.0);

                    // Comandos sugeridos
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
                                    self.modal_app.commands.push(suggestion.to_string());
                                }
                            }
                        });
                    });

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Botões de ação
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("❌ Cancelar").size(14.0))
                            .clicked()
                        {
                            self.close_modal();
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let save_enabled = !self.modal_app.name.trim().is_empty();
                            
                            ui.add_enabled_ui(save_enabled, |ui| {
                                let save_text = if is_editing { "💾 Salvar" } else { "✅ Criar" };
                                if ui
                                    .button(
                                        egui::RichText::new(save_text)
                                            .size(14.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .clicked()
                                {
                                    self.save_app();
                                }
                            });

                            if !save_enabled {
                                ui.label(
                                    egui::RichText::new("⚠ Nome é obrigatório")
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(255, 200, 100)),
                                );
                            }
                        });
                    });
                });
        }
    }

    fn render_delete_confirm(&mut self, ctx: &egui::Context) {
        if let Some(index) = self.show_delete_confirm {
            let app_name = self.state.apps.get(index).map(|a| a.name.clone()).unwrap_or_default();
            
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
                            self.show_delete_confirm = None;
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(egui::RichText::new("🗑 Excluir").color(egui::Color32::from_rgb(255, 100, 100))).clicked() {
                                self.state.apps.remove(index);
                                self.save_state();
                                self.show_delete_confirm = None;
                            }
                        });
                    });
                });
        }
    }

    fn close_modal(&mut self) {
        self.show_add_modal = false;
        self.show_edit_modal = false;
        self.show_icon_picker = false;
        self.edit_index = None;
        self.modal_app = AppConfig::default();
        self.new_command.clear();
        self.icon_search_filter.clear();
    }

    fn save_app(&mut self) {
        self.modal_app.name = self.modal_app.name.trim().to_string();
        self.modal_app.working_dir = self.modal_app.working_dir.trim().to_string();

        if let Some(index) = self.edit_index {
            self.state.apps[index] = self.modal_app.clone();
        } else {
            self.state.apps.push(self.modal_app.clone());
        }

        self.save_state();
        self.close_modal();
    }

    fn export_config(&self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Exportar configurações do Iris")
            .set_file_name("iris-config.json")
            .add_filter("JSON", &["json"])
            .save_file()
        {
            if let Ok(json) = serde_json::to_string_pretty(&self.state) {
                if let Err(e) = fs::write(&path, json) {
                    eprintln!("Erro ao exportar configurações: {}", e);
                }
            }
        }
    }

    fn import_config(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Importar configurações do Iris")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            if let Ok(content) = fs::read_to_string(&path) {
                match serde_json::from_str::<AppState>(&content) {
                    Ok(mut imported_state) => {
                        // Gera novos IDs para evitar conflitos
                        for app in &mut imported_state.apps {
                            app.id = uuid_simple();
                        }
                        
                        // Adiciona as apps importadas às existentes
                        self.state.apps.extend(imported_state.apps);
                        self.save_state();
                    }
                    Err(e) => {
                        eprintln!("Erro ao importar configurações: {}", e);
                    }
                }
            }
        }
    }
}

impl eframe::App for AppHub {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Configurar estilo moderno
        let mut visuals = egui::Visuals::dark();
        visuals.window_rounding = egui::Rounding::same(12.0);
        visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::vec2(0.0, 8.0),
            blur: 24.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(100),
        };
        visuals.popup_shadow = visuals.window_shadow;
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(28, 28, 32);
        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(45, 45, 50);
        visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(45, 45, 50);
        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(60, 60, 68);
        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(70, 70, 78);
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 120, 215);
        visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 24);
        visuals.faint_bg_color = egui::Color32::from_rgb(35, 35, 40);
        ctx.set_visuals(visuals);
        
        // Configurar fontes e espaçamentos
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
        ctx.set_style(style);
        
        // Limpar processos mortos periodicamente
        self.cleanup_dead_processes();
        
        // Verificar se tem algo em loading para animação
        let has_loading = {
            let loading = self.loading_apps.lock().unwrap();
            !loading.is_empty()
        };
        
        // Contar running para verificar estado
        let has_running = {
            let running = self.running_apps.lock().unwrap();
            !running.is_empty()
        };
        
        // Solicitar repaint - mais rápido se tiver loading ou running (para animação)
        if has_loading || has_running {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_secs(2));
        }

        // Painel superior (header) - Design moderno
        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(22, 22, 26))
                .inner_margin(egui::Margin::symmetric(20.0, 16.0))
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo e título com gradiente visual
                    ui.heading(
                        egui::RichText::new("🌈")
                            .size(32.0),
                    );
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.add_space(2.0);
                        ui.label(
                            egui::RichText::new("Iris")
                                .size(24.0)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                        ui.label(
                            egui::RichText::new("Mensageira dos Devs")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(140, 140, 150)),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Botão Nova Aplicação - Estilo moderno
                        let add_btn = egui::Button::new(
                            egui::RichText::new("➕  Nova Aplicação")
                                .size(14.0)
                                .color(egui::Color32::WHITE),
                        )
                        .fill(egui::Color32::from_rgb(99, 102, 241)) // Indigo
                        .rounding(10.0)
                        .min_size(egui::vec2(140.0, 38.0));
                        
                        if ui.add(add_btn).clicked() {
                            self.modal_app = AppConfig {
                                id: uuid_simple(),
                                icon_emoji: String::new(),
                                ..Default::default()
                            };
                            self.show_add_modal = true;
                        }

                        ui.add_space(8.0);

                        // Menu de importar/exportar
                        ui.menu_button(
                            egui::RichText::new("⚙")
                                .size(16.0)
                                .color(egui::Color32::from_rgb(180, 180, 190)),
                            |ui| {
                                ui.set_min_width(160.0);
                                
                                if ui.button("📤  Exportar configurações").clicked() {
                                    self.export_config();
                                    ui.close_menu();
                                }
                                
                                if ui.button("📥  Importar configurações").clicked() {
                                    self.import_config();
                                    ui.close_menu();
                                }
                                
                                ui.separator();
                                
                                ui.label(
                                    egui::RichText::new("Compartilhe suas configurações!")
                                        .size(10.0)
                                        .color(egui::Color32::from_rgb(120, 120, 130))
                                );
                            }
                        );

                        ui.add_space(16.0);

                        // Campo de busca - Estilo moderno
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(38, 38, 45))
                            .rounding(10.0)
                            .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("🔍")
                                            .size(14.0)
                                            .color(egui::Color32::from_rgb(120, 120, 130)),
                                    );
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.search_filter)
                                            .desired_width(180.0)
                                            .frame(false)
                                            .hint_text(
                                                egui::RichText::new("Buscar aplicações...")
                                                    .color(egui::Color32::from_rgb(100, 100, 110))
                                            ),
                                    );
                                });
                            });
                    });
                });
            });

        // Contar processos rodando
        let running_count = {
            let running = self.running_apps.lock().unwrap();
            running.len()
        };

        // Painel inferior (footer) - Design moderno
        egui::TopBottomPanel::bottom("footer")
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(22, 22, 26))
                .inner_margin(egui::Margin::symmetric(20.0, 12.0))
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Badge de contagem
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(38, 38, 45))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(format!("📦 {} apps", self.state.apps.len()))
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(160, 160, 170)),
                            );
                        });
                    
                    ui.add_space(12.0);
                    
                    if running_count > 0 {
                        // Badge de execução com animação
                        egui::Frame::none()
                            .fill(egui::Color32::from_rgb(34, 55, 40))
                            .rounding(6.0)
                            .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("● {} executando", running_count))
                                        .size(12.0)
                                        .color(egui::Color32::from_rgb(80, 220, 120)),
                                );
                            });
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new("Feito com ❤️ em Rust")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(80, 80, 90)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("•")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(60, 60, 70)),
                        );
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("v1.0.0")
                                .size(11.0)
                                .color(egui::Color32::from_rgb(80, 80, 90)),
                        );
                    });
                });
            });

        // Área central - Design moderno
        egui::CentralPanel::default()
            .frame(egui::Frame::none()
                .fill(egui::Color32::from_rgb(18, 18, 22))
                .inner_margin(egui::Margin::same(24.0))
            )
            .show(ctx, |ui| {
            if self.state.apps.is_empty() {
                // Estado vazio - Design moderno
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    
                    // Ícone grande com container
                    egui::Frame::none()
                        .fill(egui::Color32::from_rgb(30, 30, 38))
                        .rounding(24.0)
                        .inner_margin(egui::Margin::same(32.0))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new("🌈")
                                    .size(56.0),
                            );
                        });
                    
                    ui.add_space(28.0);
                    
                    ui.label(
                        egui::RichText::new("Bem-vindo ao Iris!")
                            .size(28.0)
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    
                    ui.add_space(8.0);
                    
                    ui.label(
                        egui::RichText::new("Seu hub de aplicações está vazio")
                            .size(16.0)
                            .color(egui::Color32::from_rgb(140, 140, 150)),
                    );
                    
                    ui.add_space(24.0);
                    
                    // Botão de criar primeira app
                    let btn = egui::Button::new(
                        egui::RichText::new("➕  Criar primeira aplicação")
                            .size(15.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(99, 102, 241))
                    .rounding(12.0)
                    .min_size(egui::vec2(220.0, 44.0));
                    
                    if ui.add(btn).clicked() {
                        self.modal_app = AppConfig {
                            id: uuid_simple(),
                            icon_emoji: String::new(),
                            ..Default::default()
                        };
                        self.show_add_modal = true;
                    }
                    
                    ui.add_space(16.0);
                    
                    ui.label(
                        egui::RichText::new("Configure comandos, escolha um ícone e execute com um clique!")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(100, 100, 110)),
                    );
                });
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(20.0);

                    // Filtrar apps pela busca
                    let filtered_indices: Vec<usize> = self
                        .state
                        .apps
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
                        ui.vertical_centered(|ui| {
                            ui.add_space(60.0);
                            
                            egui::Frame::none()
                                .fill(egui::Color32::from_rgb(28, 28, 35))
                                .rounding(16.0)
                                .inner_margin(egui::Margin::same(24.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new("🔍")
                                            .size(32.0),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new("Nenhuma aplicação encontrada")
                                            .size(16.0)
                                            .color(egui::Color32::from_rgb(160, 160, 170)),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        egui::RichText::new(format!("Nenhum resultado para \"{}\".", self.search_filter))
                                            .size(13.0)
                                            .color(egui::Color32::from_rgb(100, 100, 110)),
                                    );
                                });
                        });
                    } else {
                        // Grid de cards responsivo
                        let available_width = ui.available_width();
                        let card_width = 260.0;
                        let spacing = 16.0;
                        let cards_per_row = ((available_width + spacing) / (card_width + spacing)).floor() as usize;
                        let cards_per_row = cards_per_row.max(1);

                        let mut app_to_launch: Option<usize> = None;
                        let mut app_to_stop: Option<usize> = None;
                        let mut app_to_restart: Option<usize> = None;

                        egui::Grid::new("apps_grid")
                            .spacing([spacing, spacing])
                            .show(ui, |ui| {
                                for (col, &index) in filtered_indices.iter().enumerate() {
                                    let (start, stop, restart) = self.render_app_card(ui, index);
                                    
                                    if start {
                                        app_to_launch = Some(index);
                                    }
                                    if stop {
                                        app_to_stop = Some(index);
                                    }
                                    if restart {
                                        app_to_restart = Some(index);
                                    }

                                    if (col + 1) % cards_per_row == 0 {
                                        ui.end_row();
                                    }
                                }
                        });

                        // Executar ações
                        if let Some(index) = app_to_launch {
                            let app = self.state.apps[index].clone();
                            self.launch_app(&app);
                        }
                        if let Some(index) = app_to_stop {
                            let app_id = self.state.apps[index].id.clone();
                            self.stop_app(&app_id);
                        }
                        if let Some(index) = app_to_restart {
                            let app = self.state.apps[index].clone();
                            self.restart_app(&app);
                        }
                    }

                    ui.add_space(20.0);
                });
            }
        });

        // Renderizar modais
        self.render_modal(ctx);
        self.render_delete_confirm(ctx);
    }
}

/// Trunca um caminho para exibição
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

fn load_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/logo.png");
    let image = image::load_from_memory(icon_bytes).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}

fn main() -> eframe::Result<()> {
    let icon = load_icon();
    
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 600.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("🌈 Iris");
    
    if let Some(icon_data) = icon {
        viewport = viewport.with_icon(Arc::new(icon_data));
    }
    
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Iris",
        options,
        Box::new(|cc| Ok(Box::new(AppHub::new(cc)))),
    )
}
