use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("embedded_icons.rs");
    
    let mut code = String::new();
    code.push_str("// Auto-generated file - DO NOT EDIT\n\n");
    code.push_str("pub fn get_embedded_icons() -> std::collections::HashMap<&'static str, &'static str> {\n");
    code.push_str("    let mut icons = std::collections::HashMap::new();\n");
    
    let icons_dir = Path::new("assets/langs");
    if icons_dir.exists() {
        if let Ok(entries) = fs::read_dir(icons_dir) {
            for entry in entries.flatten() {
                let filename = entry.file_name().to_string_lossy().to_string();
                if filename.ends_with("-original.svg") {
                    let name = filename
                        .strip_suffix("-original.svg")
                        .unwrap_or(&filename);
                    
                    let rel_path = format!("assets/langs/{}", filename);
                    code.push_str(&format!(
                        "    icons.insert(\"{}\", include_str!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/{}\")) );\n",
                        name, rel_path
                    ));
                }
            }
        }
    }
    
    code.push_str("    icons\n");
    code.push_str("}\n");
    
    fs::write(&dest_path, code).unwrap();
    
    println!("cargo:rerun-if-changed=assets/langs/");
    println!("cargo:rerun-if-changed=build.rs");
}
