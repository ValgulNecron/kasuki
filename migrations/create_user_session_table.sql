-- Create user_session table for OAuth session management
-- This table stores user session tokens and Discord OAuth tokens

CREATE TABLE IF NOT EXISTS user_session (
    session_token VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    discord_access_token TEXT NOT NULL,
    discord_refresh_token TEXT NOT NULL,
    token_expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Index on user_id for faster lookups
    INDEX idx_user_session_user_id (user_id),
    
    -- Index on last_used_at for cleanup queries
    INDEX idx_user_session_last_used (last_used_at)
);

-- Optional: Add a comment describing the table
COMMENT ON TABLE user_session IS 'Stores user session tokens and Discord OAuth credentials for the web API';
