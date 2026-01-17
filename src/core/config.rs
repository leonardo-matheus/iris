//! # Gerenciamento de Configurações
//! 
//! Este módulo é responsável por carregar, salvar e gerenciar
//! as configurações da aplicação Iris.
//! 
//! ## Localização do Arquivo de Configuração
//! As configurações são salvas em: `%APPDATA%\iris\config.json`

use std::fs;
use std::path::PathBuf;
use super::models::AppState;

/// Gerenciador de configurações da aplicação.
/// 
/// Responsável por:
/// - Determinar o caminho do arquivo de configuração
/// - Carregar configurações do disco
/// - Salvar configurações em disco
/// - Importar/Exportar configurações
pub struct ConfigManager {
    /// Caminho do arquivo de configuração
    config_path: PathBuf,
}

impl ConfigManager {
    /// Cria uma nova instância do gerenciador de configurações.
    /// 
    /// Automaticamente determina o caminho correto baseado no sistema operacional.
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        Self { config_path }
    }
    
    /// Retorna o caminho do arquivo de configuração.
    /// 
    /// No Windows: `%APPDATA%\iris\config.json`
    /// No Linux/macOS: `~/.config/iris/config.json`
    pub fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("iris");
        fs::create_dir_all(&path).ok();
        path.push("config.json");
        path
    }
    
    /// Retorna uma referência ao caminho do arquivo de configuração
    pub fn path(&self) -> &PathBuf {
        &self.config_path
    }
    
    /// Carrega o estado da aplicação do disco.
    /// 
    /// Se o arquivo não existir ou for inválido, retorna um estado vazio.
    /// 
    /// # Exemplo
    /// ```rust
    /// let manager = ConfigManager::new();
    /// let state = manager.load();
    /// println!("Aplicações carregadas: {}", state.apps.len());
    /// ```
    pub fn load(&self) -> AppState {
        if self.config_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.config_path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        AppState::default()
    }
    
    /// Salva o estado da aplicação em disco.
    /// 
    /// # Argumentos
    /// * `state` - Estado a ser salvo
    /// 
    /// # Retorno
    /// Retorna `Ok(())` em caso de sucesso ou `Err` com a mensagem de erro.
    pub fn save(&self, state: &AppState) -> Result<(), String> {
        match serde_json::to_string_pretty(state) {
            Ok(json) => {
                fs::write(&self.config_path, json)
                    .map_err(|e| format!("Erro ao salvar configurações: {}", e))
            }
            Err(e) => Err(format!("Erro ao serializar configurações: {}", e)),
        }
    }
    
    /// Exporta as configurações para um arquivo específico.
    /// 
    /// # Argumentos
    /// * `state` - Estado a ser exportado
    /// * `path` - Caminho do arquivo de destino
    pub fn export(&self, state: &AppState, path: &PathBuf) -> Result<(), String> {
        match serde_json::to_string_pretty(state) {
            Ok(json) => {
                fs::write(path, json)
                    .map_err(|e| format!("Erro ao exportar configurações: {}", e))
            }
            Err(e) => Err(format!("Erro ao serializar configurações: {}", e)),
        }
    }
    
    /// Importa configurações de um arquivo.
    /// 
    /// As aplicações importadas recebem novos IDs para evitar conflitos.
    /// 
    /// # Argumentos
    /// * `path` - Caminho do arquivo a ser importado
    /// 
    /// # Retorno
    /// Retorna o estado importado ou uma mensagem de erro.
    pub fn import(&self, path: &PathBuf) -> Result<AppState, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Erro ao ler arquivo: {}", e))?;
        
        let mut state: AppState = serde_json::from_str(&content)
            .map_err(|e| format!("Erro ao processar JSON: {}", e))?;
        
        // Gera novos IDs para evitar conflitos
        for app in &mut state.apps {
            app.id = crate::utils::uuid_simple();
        }
        
        Ok(state)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_path_exists() {
        let path = ConfigManager::get_config_path();
        assert!(path.to_string_lossy().contains("iris"));
    }
    
    #[test]
    fn test_default_state_is_empty() {
        let state = AppState::default();
        assert!(state.apps.is_empty());
    }
}
