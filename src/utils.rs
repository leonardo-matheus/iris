//! # Utils Module
//! 
//! Funções utilitárias usadas em toda a aplicação.

/// Gera um ID simples baseado em timestamp.
/// 
/// Combina segundos desde UNIX epoch com nanosegundos
/// para criar um identificador único.
/// 
/// # Exemplo
/// ```rust
/// let id = uuid_simple();
/// assert!(!id.is_empty());
/// ```
pub fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}{}", duration.as_secs(), duration.subsec_nanos())
}

/// Trunca um caminho para exibição.
/// 
/// Se o caminho for maior que o tamanho máximo,
/// adiciona "..." no início e mostra apenas o final.
/// 
/// # Argumentos
/// * `path` - Caminho a ser truncado
/// * `max_len` - Tamanho máximo em caracteres
/// 
/// # Exemplo
/// ```rust
/// let truncated = truncate_path("C:\\Users\\Name\\Projects\\MyApp", 20);
/// assert!(truncated.len() <= 20);
/// ```
pub fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len() - max_len + 3..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_uuid_simple_not_empty() {
        let id = uuid_simple();
        assert!(!id.is_empty());
    }
    
    #[test]
    fn test_uuid_simple_unique() {
        let id1 = uuid_simple();
        std::thread::sleep(std::time::Duration::from_nanos(1));
        let id2 = uuid_simple();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_truncate_path_short() {
        let path = "C:\\short";
        let result = truncate_path(path, 20);
        assert_eq!(result, path);
    }
    
    #[test]
    fn test_truncate_path_long() {
        let path = "C:\\Users\\Name\\Very\\Long\\Path\\To\\Project";
        let result = truncate_path(path, 20);
        assert!(result.starts_with("..."));
        assert!(result.len() <= 20);
    }
}
