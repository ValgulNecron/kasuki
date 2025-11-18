fn main() {
    let mut api_url = None;
    
    // Load .env file if it exists
    if let Ok(path) = std::env::current_dir() {
        let env_path = path.join(".env");
        if env_path.exists() {
            eprintln!("Loading .env from: {}", env_path.display());
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
                        let key = key.trim();
                        let value = value.trim();
                        
                        // Store KASUKI_API_URL for validation
                        if key == "KASUKI_API_URL" {
                            api_url = Some(value.to_string());
                            eprintln!("Found KASUKI_API_URL in .env: {}", value);
                        }
                        
                        println!("cargo:rustc-env={}={}", key, value);
                    }
                }
            }
        } else {
            eprintln!("No .env file found at: {}", env_path.display());
        }
    }
    
    // Set default if not provided
    let final_url = api_url.unwrap_or_else(|| {
        eprintln!("KASUKI_API_URL not found in .env, using default: http://localhost:8080");
        "http://localhost:8080".to_string()
    });
    
    // Always set the value (will use the one from .env or the default)
    if std::env::var("KASUKI_API_URL").is_err() {
        println!("cargo:rustc-env=KASUKI_API_URL={}", final_url);
    }
    
    eprintln!("Build-time KASUKI_API_URL: {}", final_url);
    
    // Rerun if .env changes
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=build.rs");
}
