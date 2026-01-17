//! # Process Manager Service
//! 
//! Este serviço é responsável pelo gerenciamento do ciclo de vida
//! dos processos das aplicações configuradas.
//! 
//! ## Funcionalidades
//! - Iniciar processos em terminais Windows
//! - Parar processos em execução
//! - Reiniciar processos
//! - Monitorar estado dos processos
//! - Limpeza automática de processos mortos

use std::collections::{HashMap, HashSet};
use std::fs;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::core::{AppConfig, RunningProcess};

/// Flags de criação do Windows para ocultar janelas de comando
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Gerenciador de Processos.
/// 
/// Controla o ciclo de vida de todos os processos das aplicações.
/// Thread-safe através de Arc<Mutex>.
/// 
/// # Exemplo
/// ```rust
/// let manager = ProcessManager::new();
/// manager.launch_app(&app_config);
/// 
/// if manager.is_running(&app_config.id) {
///     manager.stop_app(&app_config.id);
/// }
/// ```
pub struct ProcessManager {
    /// Mapa de processos em execução (app_id -> RunningProcess)
    running_apps: Arc<Mutex<HashMap<String, RunningProcess>>>,
    
    /// Conjunto de aplicações em processo de inicialização
    loading_apps: Arc<Mutex<HashSet<String>>>,
}

impl ProcessManager {
    /// Cria uma nova instância do gerenciador de processos
    pub fn new() -> Self {
        Self {
            running_apps: Arc::new(Mutex::new(HashMap::new())),
            loading_apps: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    /// Retorna uma referência Arc para os processos em execução
    pub fn running_apps(&self) -> Arc<Mutex<HashMap<String, RunningProcess>>> {
        Arc::clone(&self.running_apps)
    }
    
    /// Retorna uma referência Arc para as aplicações em loading
    pub fn loading_apps(&self) -> Arc<Mutex<HashSet<String>>> {
        Arc::clone(&self.loading_apps)
    }
    
    /// Inicia uma aplicação em um novo terminal Windows.
    /// 
    /// Este método:
    /// 1. Para qualquer processo anterior da mesma aplicação
    /// 2. Marca a aplicação como "loading"
    /// 3. Cria um arquivo batch temporário com os comandos
    /// 4. Executa o batch em um novo terminal
    /// 5. Registra o processo em execução
    /// 
    /// # Argumentos
    /// * `app` - Configuração da aplicação a ser iniciada
    pub fn launch_app(&self, app: &AppConfig) {
        if app.commands.is_empty() {
            return;
        }

        // Para o processo anterior se existir
        self.stop_app(&app.id, Some(&app.name), Some(&app.commands));
        
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
            Self::launch_in_thread(app_clone, running_apps, loading_apps);
        });
    }
    
    /// Lógica de inicialização executada em thread separada
    fn launch_in_thread(
        app: AppConfig,
        running_apps: Arc<Mutex<HashMap<String, RunningProcess>>>,
        loading_apps: Arc<Mutex<HashSet<String>>>,
    ) {
        // Cria um arquivo batch temporário
        let temp_dir = std::env::temp_dir();
        let batch_file = temp_dir.join(format!("iris_{}.bat", app.id));
        
        // Monta o conteúdo do batch
        let batch_content = Self::build_batch_content(&app);
        
        if fs::write(&batch_file, &batch_content).is_err() {
            let mut loading = loading_apps.lock().unwrap();
            loading.remove(&app.id);
            return;
        }

        // Executa o batch
        let child = Command::new("cmd")
            .args(["/C", "start", "", &batch_file.to_string_lossy()])
            .spawn();

        // Aguarda um pouco para o console abrir
        std::thread::sleep(Duration::from_millis(800));

        if let Ok(child) = child {
            // Tenta obter o PID do processo do console
            let console_pid = Self::find_console_pid(&app.name);
            
            let mut running = running_apps.lock().unwrap();
            running.insert(app.id.clone(), RunningProcess::new(child, console_pid));
        }
        
        // Remove do loading
        let mut loading = loading_apps.lock().unwrap();
        loading.remove(&app.id);
    }
    
    /// Constrói o conteúdo do arquivo batch para execução.
    /// 
    /// Trata comandos especiais como npm, yarn, cargo e scripts .bat.
    /// Também detecta automaticamente inputs para scripts interativos.
    fn build_batch_content(app: &AppConfig) -> String {
        let mut batch_content = String::new();
        batch_content.push_str("@echo off\n");
        batch_content.push_str(&format!("title [IRIS] {}\n", app.name));
        
        if !app.working_dir.is_empty() {
            batch_content.push_str(&format!("cd /d \"{}\"\n", app.working_dir));
        }
        
        let commands = &app.commands;
        let mut i = 0;
        while i < commands.len() {
            let cmd = &commands[i];
            let cmd_lower = cmd.to_lowercase();
            
            // Verifica se o próximo comando parece ser um input
            let next_is_input = if i + 1 < commands.len() {
                let next = &commands[i + 1];
                next.chars().all(|c| c.is_numeric() || c == '.')
                    || next.eq_ignore_ascii_case("s")
                    || next.eq_ignore_ascii_case("n")
                    || next.eq_ignore_ascii_case("y")
            } else {
                false
            };
            
            // Se o comando atual é um .bat/.cmd e o próximo é input
            if next_is_input && (cmd_lower.ends_with(".bat") || cmd_lower.ends_with(".cmd")) {
                let input = &commands[i + 1];
                let input_file = format!("iris_input_{}.txt", app.id);
                batch_content.push_str(&format!("echo {}> %TEMP%\\{}\n", input, input_file));
                batch_content.push_str(&format!("call {} < %TEMP%\\{}\n", cmd, input_file));
                batch_content.push_str(&format!("del %TEMP%\\{} 2>nul\n", input_file));
                i += 2;
                batch_content.push_str(&format!("title [IRIS] {}\n", app.name));
                continue;
            }
            
            // Comandos que precisam de "call"
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
            
            batch_content.push_str(&format!("title [IRIS] {}\n", app.name));
            i += 1;
        }
        
        batch_content.push_str(&format!("title [IRIS] {}\n", app.name));
        batch_content.push_str("cmd /k\n");
        
        batch_content
    }
    
    /// Encontra o PID do console Windows pelo título da janela.
    fn find_console_pid(title: &str) -> Option<u32> {
        let search_title = format!("[IRIS] {}", title);
        
        for _ in 0..5 {
            // Busca pelo título exato
            #[cfg(windows)]
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Get-Process cmd -ErrorAction SilentlyContinue | Where-Object {{$_.MainWindowTitle -eq '{}'}} | Select-Object -First 1 -ExpandProperty Id",
                        search_title
                    ),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()?;
            
            #[cfg(not(windows))]
            let output = Command::new("echo")
                .arg("")
                .output()
                .ok()?;

            let pid_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(pid) = pid_str.trim().parse() {
                return Some(pid);
            }
            
            // Tenta com -like se -eq não funcionou
            #[cfg(windows)]
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "Get-Process cmd -ErrorAction SilentlyContinue | Where-Object {{$_.MainWindowTitle -like '*[IRIS]*{}*'}} | Select-Object -First 1 -ExpandProperty Id",
                        title
                    ),
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()?;

            #[cfg(not(windows))]
            let output = Command::new("echo")
                .arg("")
                .output()
                .ok()?;

            let pid_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(pid) = pid_str.trim().parse() {
                return Some(pid);
            }
            
            std::thread::sleep(Duration::from_millis(300));
        }
        
        None
    }
    
    /// Para uma aplicação em execução.
    /// 
    /// Utiliza múltiplas estratégias para garantir que o processo seja terminado:
    /// 1. Mata pelo título do comando
    /// 2. Mata pelo título [IRIS]
    /// 3. Mata pela árvore de processos (PID)
    /// 4. Usa WMIC para matar pelo CommandLine
    /// 
    /// # Argumentos
    /// * `app_id` - ID da aplicação a ser parada
    /// * `app_name` - Nome opcional da aplicação (para busca por título)
    /// * `commands` - Comandos opcionais (para busca por título)
    pub fn stop_app(&self, app_id: &str, app_name: Option<&str>, commands: Option<&Vec<String>>) {
        let mut running = self.running_apps.lock().unwrap();
        
        if let Some(mut process) = running.remove(app_id) {
            // Estratégia 1: Mata pelo título do comando
            if let Some(cmds) = commands {
                for cmd in cmds {
                    #[cfg(windows)]
                    {
                        let _ = Command::new("taskkill")
                            .args(["/F", "/FI", &format!("WINDOWTITLE eq {}", cmd)])
                            .creation_flags(CREATE_NO_WINDOW)
                            .output();
                        
                        let _ = Command::new("taskkill")
                            .args(["/F", "/FI", &format!("WINDOWTITLE eq {}*", cmd)])
                            .creation_flags(CREATE_NO_WINDOW)
                            .output();
                    }
                }
            }

            // Estratégia 2: Pelo título [IRIS] Nome
            if let Some(name) = app_name {
                #[cfg(windows)]
                let _ = Command::new("taskkill")
                    .args(["/F", "/FI", &format!("WINDOWTITLE eq [IRIS] {}", name)])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }

            // Estratégia 3: Pela árvore de processos
            if let Some(pid) = process.console_pid {
                #[cfg(windows)]
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }

            // Estratégia 4: WMIC pelo CommandLine
            #[cfg(windows)]
            {
                let batch_name = format!("iris_{}.bat", app_id);
                let _ = Command::new("cmd")
                    .args(["/C", &format!(
                        "wmic process where \"CommandLine like '%{}%'\" call terminate 2>nul",
                        batch_name
                    )])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }

            // Estratégia 5: Mata padrões comuns
            #[cfg(windows)]
            for pattern in ["npm*", "node*", "vite*", "yarn*", "pnpm*"] {
                let _ = Command::new("taskkill")
                    .args(["/F", "/FI", &format!("WINDOWTITLE eq {}", pattern)])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();
            }
            
            // Mata o processo child diretamente
            let _ = process.child.kill();
        }
    }
    
    /// Reinicia uma aplicação.
    /// 
    /// Para o processo atual e inicia novamente após um pequeno delay.
    pub fn restart_app(&self, app: &AppConfig) {
        self.stop_app(&app.id, Some(&app.name), Some(&app.commands));
        std::thread::sleep(Duration::from_millis(200));
        self.launch_app(app);
    }
    
    /// Verifica se uma aplicação está em execução
    pub fn is_running(&self, app_id: &str) -> bool {
        let running = self.running_apps.lock().unwrap();
        running.contains_key(app_id)
    }
    
    /// Verifica se uma aplicação está em processo de inicialização
    pub fn is_loading(&self, app_id: &str) -> bool {
        let loading = self.loading_apps.lock().unwrap();
        loading.contains(app_id)
    }
    
    /// Retorna o número de aplicações em execução
    pub fn running_count(&self) -> usize {
        let running = self.running_apps.lock().unwrap();
        running.len()
    }
    
    /// Verifica se há alguma aplicação em loading
    pub fn has_loading(&self) -> bool {
        let loading = self.loading_apps.lock().unwrap();
        !loading.is_empty()
    }
    
    /// Verifica se há alguma aplicação em execução
    pub fn has_running(&self) -> bool {
        let running = self.running_apps.lock().unwrap();
        !running.is_empty()
    }
    
    /// Limpa processos que morreram do registro.
    /// 
    /// Verifica se os processos registrados ainda estão ativos
    /// e remove os que foram encerrados.
    pub fn cleanup_dead_processes(&self) {
        let mut running = self.running_apps.lock().unwrap();
        let mut to_remove = Vec::new();
        
        for (app_id, process) in running.iter() {
            if let Some(pid) = process.console_pid {
                #[cfg(windows)]
                let output = Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                    .output();
                
                #[cfg(not(windows))]
                let output = Command::new("ps")
                    .args(["-p", &pid.to_string()])
                    .output();
                
                if let Ok(output) = output {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    #[cfg(windows)]
                    let is_dead = !output_str.to_lowercase().contains("cmd.exe");
                    #[cfg(not(windows))]
                    let is_dead = output_str.is_empty();
                    
                    if is_dead {
                        to_remove.push(app_id.clone());
                    }
                }
            }
        }
        
        for app_id in to_remove {
            running.remove(&app_id);
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
