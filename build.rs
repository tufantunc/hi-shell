use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=.env");
    let env_path = Path::new(".env");

    if env_path.exists() {
        let content = fs::read_to_string(env_path).expect("Failed to read .env file");
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                println!("cargo:rustc-env={}={}", key.trim(), value.trim());
            }
        }
    }
}
