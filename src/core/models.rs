//! # Modelos de Dados
//! 
//! Define todas as estruturas de dados utilizadas pela aplicação Iris.
//! Estas estruturas são serializáveis para persistência em JSON.

use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::process::Child;

/// Estrutura que representa uma aplicação configurada pelo usuário.
/// 
/// Cada aplicação possui um identificador único, nome, ícone,
/// diretório de trabalho e uma lista de comandos a serem executados.
/// 
/// # Exemplo
/// ```rust
/// let app = AppConfig {
///     id: "123456789".to_string(),
///     name: "Minha App React".to_string(),
///     icon_emoji: "react".to_string(),
///     working_dir: "C:\\projetos\\minha-app".to_string(),
///     commands: vec!["npm install".to_string(), "npm run dev".to_string()],
/// };
/// ```
#[derive(Clone, Serialize, Deserialize, Default, Debug)]
pub struct AppConfig {
    /// Identificador único da aplicação (gerado automaticamente)
    pub id: String,
    
    /// Nome da aplicação exibido na interface
    pub name: String,
    
    /// Nome do ícone da tecnologia (ex: "react", "python", "docker")
    pub icon_emoji: String,
    
    /// Diretório de trabalho onde os comandos serão executados
    pub working_dir: String,
    
    /// Lista de comandos a serem executados em sequência
    pub commands: Vec<String>,
}

impl AppConfig {
    /// Cria uma nova configuração de aplicação com ID gerado automaticamente
    pub fn new(name: String) -> Self {
        Self {
            id: crate::utils::uuid_simple(),
            name,
            ..Default::default()
        }
    }
    
    /// Verifica se a aplicação tem comandos configurados
    pub fn has_commands(&self) -> bool {
        !self.commands.is_empty()
    }
    
    /// Retorna o número de comandos configurados
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

/// Estado global da aplicação.
/// 
/// Contém a lista de todas as aplicações configuradas pelo usuário.
/// Este estado é persistido em disco automaticamente.
#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    /// Lista de aplicações configuradas
    pub apps: Vec<AppConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        Self { apps: Vec::new() }
    }
}

impl AppState {
    /// Adiciona uma nova aplicação ao estado
    pub fn add_app(&mut self, app: AppConfig) {
        self.apps.push(app);
    }
    
    /// Remove uma aplicação pelo índice
    pub fn remove_app(&mut self, index: usize) -> Option<AppConfig> {
        if index < self.apps.len() {
            Some(self.apps.remove(index))
        } else {
            None
        }
    }
    
    /// Busca uma aplicação pelo ID
    pub fn find_by_id(&self, id: &str) -> Option<&AppConfig> {
        self.apps.iter().find(|app| app.id == id)
    }
    
    /// Busca uma aplicação mutável pelo ID
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut AppConfig> {
        self.apps.iter_mut().find(|app| app.id == id)
    }
    
    /// Retorna o número total de aplicações
    pub fn app_count(&self) -> usize {
        self.apps.len()
    }
}

/// Informações de um processo em execução.
/// 
/// Mantém referência ao processo filho e informações
/// adicionais como PID do console e tempo de início.
pub struct RunningProcess {
    /// Processo filho do Rust
    #[allow(dead_code)]
    pub child: Child,
    
    /// PID do console Windows (cmd.exe)
    pub console_pid: Option<u32>,
    
    /// Momento em que o processo foi iniciado
    #[allow(dead_code)]
    pub started_at: Instant,
}

impl RunningProcess {
    /// Cria um novo registro de processo em execução
    pub fn new(child: Child, console_pid: Option<u32>) -> Self {
        Self {
            child,
            console_pid,
            started_at: Instant::now(),
        }
    }
}

/// Informações sobre um ícone disponível.
/// 
/// Os ícones são arquivos SVG armazenados em `assets/langs/`.
#[derive(Clone, Debug)]
pub struct IconInfo {
    /// Nome do ícone (ex: "react", "python")
    pub name: String,
    
    /// Nome do arquivo (ex: "react-original.svg")
    #[allow(dead_code)]
    pub filename: String,
}

impl IconInfo {
    /// Cria uma nova informação de ícone
    pub fn new(name: String, filename: String) -> Self {
        Self { name, filename }
    }
}
