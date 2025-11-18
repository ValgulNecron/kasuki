/// Configuration loaded at compile time from environment variables
pub struct Config;

impl Config {
    /// Get the API base URL
    /// This is loaded from the KASUKI_API_URL environment variable at compile time
    pub fn api_url() -> &'static str {
        env!("KASUKI_API_URL")
    }
    
    /// Get the OAuth login URL
    pub fn oauth_login_url() -> String {
        format!("{}/api/oauth/login", Self::api_url())
    }
}
