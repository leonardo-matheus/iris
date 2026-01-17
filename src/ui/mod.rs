//! # UI Module
//! 
//! Este módulo contém todos os componentes de interface gráfica
//! da aplicação Iris.
//! 
//! ## Componentes
//! - `app_hub`: Aplicação principal e estado da UI
//! - `components`: Componentes reutilizáveis (cards, modais, etc.)
//! - `theme`: Configurações de tema e estilo
//! - `dialogs`: Diálogos e modais

pub mod app_hub;
pub mod components;
pub mod theme;
pub mod dialogs;

pub use app_hub::*;
pub use theme::*;
