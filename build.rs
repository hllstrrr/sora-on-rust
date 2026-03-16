use std::fs;
use std::path::Path;

fn main() {
    let commands_path = Path::new("src/commands");
    generate_mod_rs(commands_path);
    println!("cargo:rerun-if-changed=src/commands");
}

fn generate_mod_rs(path: &Path) {
    let entries = fs::read_dir(path).unwrap();
    let mut mods = Vec::new();

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.is_dir() {
            let dir_name = path.file_name().unwrap().to_str().unwrap();
            mods.push(format!("pub mod {};", dir_name));
            generate_mod_rs(&path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            if file_name != "mod" {
                mods.push(format!("pub mod {};", file_name));
            }
        }
    }

    let mod_file = path.join("mod.rs");
    let content = mods.join("\n");
    let _ = fs::write(mod_file, content);
}