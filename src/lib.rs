//! # Iris - Mensageira dos Devs
//! 
//! Iris é um hub de aplicações para desenvolvedores que trabalham com
//! múltiplas tecnologias. Permite configurar, executar e gerenciar
//! aplicações de forma centralizada.
//! 
//! ## Arquitetura em Microserviços
//! 
//! A aplicação está organizada em módulos com responsabilidades bem definidas:
//! 
//! ### Core (`core/`)
//! Contém as estruturas de dados fundamentais:
//! - `models`: Definições de `AppConfig`, `AppState`, `RunningProcess`, etc.
//! - `config`: Gerenciamento de configurações e persistência em JSON
//! 
//! ### Services (`services/`)
//! Serviços de negócio independentes:
//! - `process_manager`: Gerenciamento do ciclo de vida de processos
//! - `icon_service`: Carregamento e cache de ícones SVG
//! 
//! ### UI (`ui/`)
//! Componentes de interface gráfica:
//! - `app_hub`: Aplicação principal e coordenação
//! - `components`: Componentes reutilizáveis (cards, header, footer)
//! - `dialogs`: Modais e diálogos (add/edit app, confirmação)
//! - `theme`: Configurações de tema e estilo
//! 
//! ### Utils (`utils.rs`)
//! Funções utilitárias compartilhadas
//! 
//! ## Exemplo de Uso
//! 
//! ```rust,no_run
//! use iris::ui::AppHub;
//! 
//! fn main() -> eframe::Result<()> {
//!     let options = eframe::NativeOptions::default();
//!     eframe::run_native(
//!         "Iris",
//!         options,
//!         Box::new(|cc| Ok(Box::new(AppHub::new(cc)))),
//!     )
//! }
//! ```

// Módulos públicos
pub mod core;
pub mod services;
pub mod ui;
pub mod utils;

/// Versão atual da aplicação
pub const VERSION: &str = "1.0.0";

// Re-exports para conveniência
pub use core::{AppConfig, AppState, ConfigManager};
pub use services::{ProcessManager, IconCache};
pub use ui::AppHub;
