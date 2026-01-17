//! # Core Module
//! 
//! Este módulo contém as estruturas de dados fundamentais do Iris.
//! Ele é a base para todos os outros microserviços da aplicação.
//! 
//! ## Componentes
//! - `models`: Definições de estruturas de dados (AppConfig, AppState, etc.)
//! - `config`: Gerenciamento de configurações e persistência

pub mod models;
pub mod config;

pub use models::*;
pub use config::*;
