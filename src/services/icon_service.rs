//! # Icon Service
//! 
//! Serviço responsável pelo carregamento, renderização e cache
//! de ícones SVG das tecnologias.
//! 
//! ## Funcionalidades
//! - Ícones SVG embutidos no executável
//! - Renderização de SVG para texturas egui
//! - Cache de texturas para performance
//! - Listagem de ícones disponíveis

use std::collections::HashMap;
use std::sync::OnceLock;
use eframe::egui::{self, ColorImage, TextureHandle};

use crate::core::IconInfo;

// Inclui os ícones gerados pelo build.rs
include!(concat!(env!("OUT_DIR"), "/embedded_icons.rs"));

// Cache estático dos ícones embutidos
static ICONS: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn get_icons() -> &'static HashMap<&'static str, &'static str> {
    ICONS.get_or_init(|| get_embedded_icons())
}

/// Cache de texturas dos ícones.
/// 
/// Armazena texturas já renderizadas para evitar
/// re-renderização a cada frame.
pub struct IconCache {
    /// Mapa de texturas (nome_do_icone -> TextureHandle)
    textures: HashMap<String, TextureHandle>,
}

impl IconCache {
    /// Cria um novo cache vazio
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }
    
    /// Obtém um ícone do cache ou carrega do disco.
    /// 
    /// Se o ícone já estiver no cache, retorna imediatamente.
    /// Caso contrário, carrega o SVG, renderiza e adiciona ao cache.
    /// 
    /// # Argumentos
    /// * `ctx` - Contexto do egui para criação de texturas
    /// * `icon_name` - Nome do ícone (ex: "react", "python")
    /// 
    /// # Retorno
    /// `Some(TextureHandle)` se o ícone foi carregado com sucesso,
    /// `None` se o ícone não existe ou falhou ao carregar.
        pub fn get_or_load(&mut self, ctx: &egui::Context, icon_name: &str) -> Option<TextureHandle> {
        // Verifica se já está no cache
        if let Some(texture) = self.textures.get(icon_name) {
            return Some(texture.clone());
        }
        
        // Busca o ícone embutido
        let icons = get_icons();
        if let Some(svg_data) = icons.get(icon_name) {
            if let Some(image) = render_svg_to_image(svg_data, 32, 32) {
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
    
    /// Limpa o cache de texturas
    pub fn clear(&mut self) {
        self.textures.clear();
    }
    
    /// Retorna o número de texturas em cache
    pub fn len(&self) -> usize {
        self.textures.len()
    }
    
    /// Verifica se o cache está vazio
    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
}

impl Default for IconCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Renderiza um SVG para uma imagem ColorImage do egui.
/// 
/// Utiliza a biblioteca resvg para renderização de alta qualidade.
/// 
/// # Argumentos
/// * `svg_data` - Conteúdo do arquivo SVG
/// * `width` - Largura desejada em pixels
/// * `height` - Altura desejada em pixels
/// 
/// # Retorno
/// `Some(ColorImage)` se a renderização foi bem sucedida,
/// `None` em caso de erro.
pub fn render_svg_to_image(svg_data: &str, width: u32, height: u32) -> Option<ColorImage> {
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

/// Carrega a lista de ícones disponíveis do diretório assets/langs.
/// 
/// Busca por arquivos com padrão `*-original.svg` e extrai o nome
/// da tecnologia do nome do arquivo.
/// 
/// # Retorno
/// Vetor de `IconInfo` ordenado alfabeticamente pelo nome.
/// 
/// # Exemplo
/// ```rust
/// let icons = load_available_icons();
/// for icon in icons {
///     println!("Ícone disponível: {}", icon.name);
/// }
/// ```
pub fn load_available_icons() -> Vec<IconInfo> {
    let icons_map = get_icons();
    let mut icons: Vec<IconInfo> = icons_map
        .keys()
        .map(|name| {
            let filename = format!("{}-original.svg", name);
            IconInfo::new(name.to_string(), filename)
        })
        .collect();
    
    icons.sort_by(|a, b| a.name.cmp(&b.name));
    icons
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_icon_cache_creation() {
        let cache = IconCache::new();
        assert!(cache.is_empty());
    }
    
    #[test]
    fn test_render_invalid_svg() {
        let result = render_svg_to_image("invalid svg content", 32, 32);
        assert!(result.is_none());
    }
}
