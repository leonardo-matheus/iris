//! # Iris - Mensageira dos Devs
//! 
//! Ponto de entrada da aplicacao.
//! 
//! Este arquivo inicializa a aplicacao GUI usando os modulos
//! da arquitetura de microservicos definidos na biblioteca.
//! 
//! ## Versao 1.0.0
//! 
//! ### Historico
//! Iris foi criada por um desenvolvedor senior que, todo dia ao chegar no trabalho,
//! precisava executar diversos comandos no terminal para iniciar sua gateway,
//! dashboard e outros projetos. Esta ferramenta centraliza e automatiza esse processo.

#![windows_subsystem = "windows"]

use std::sync::Arc;
use eframe::egui;

// Importa os modulos da aplicacao
mod core;
mod services;
mod ui;
mod utils;

/// Versao atual da aplicacao
pub const VERSION: &str = "1.0.0";

/// Carrega o icone da janela a partir do arquivo PNG embutido.
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

/// Ponto de entrada da aplicacao Iris.
/// 
/// Configura a janela nativa com icone personalizado e inicia o loop de renderizacao.
/// A aplicacao usa a arquitetura de microservicos definida nos modulos:
/// - `core`: Modelos e configuracao
/// - `services`: Gerenciamento de processos e icones
/// - `ui`: Componentes de interface
fn main() -> eframe::Result<()> {
    let icon = load_icon();
    
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 600.0])
        .with_min_inner_size([600.0, 400.0])
        .with_title("Iris - Mensageira dos Devs");
    
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
        Box::new(|cc| Ok(Box::new(ui::AppHub::new(cc)))),
    )
}