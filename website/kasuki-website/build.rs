fn main() {
    // Load .env file if it exists
    if let Ok(path) = std::env::current_dir() {
        let env_path = path.join(".env");
        if env_path.exists() {
            // Read .env file and set environment variables for the build
            if let Ok(content) = std::fs::read_to_string(&env_path) {
                for line in content.lines() {
                    let line = line.trim();
                    // Skip comments and empty lines
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    // Parse KEY=VALUE
                    if let Some((key, value)) = line.split_once('=') {
                        println!("cargo:rustc-env={}={}", key.trim(), value.trim());
                    }
                }
            }
        }
    }
    
    // Set default if not provided
    if std::env::var("KASUKI_API_URL").is_err() {
        println!("cargo:rustc-env=KASUKI_API_URL=http://localhost:8080");
    }
    
    // Rerun if .env changes
    println!("cargo:rerun-if-changed=.env");
}
