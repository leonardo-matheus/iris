//! # Services Module
//! 
//! Este módulo contém os serviços de negócio da aplicação Iris.
//! Cada serviço é responsável por uma funcionalidade específica.
//! 
//! ## Serviços Disponíveis
//! - `process_manager`: Gerenciamento de processos (start, stop, restart)
//! - `icon_service`: Carregamento e cache de ícones SVG

pub mod process_manager;
pub mod icon_service;

pub use process_manager::*;
pub use icon_service::*;
