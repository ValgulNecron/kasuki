# OAuth Token Management and WebUI Login Fix - Implementation Summary

## Problem Statement

The `/bot/api` OAuth implementation had two critical issues:
1. **No token persistence**: Discord access and refresh tokens were received but never saved
2. **No session management**: Users were redirected after login but appeared logged out on the main page

## Solution Overview

Implemented a complete session management system with:
- Database-backed session storage
- HTTP-only secure cookies
- Automatic token refresh
- Proper logout functionality

## Architecture

### Authentication Flow

```
1. User clicks "Login with Discord"
   ↓
2. Redirect to Discord OAuth
   ↓
3. User authorizes → Discord redirects to /api/oauth/callback
   ↓
4. Backend:
   - Exchange code for tokens
   - Generate session token (UUID)
   - Store in database: session_token → user_id, discord_tokens, expiry
   - Set HTTP-only cookie: session_token
   - Redirect to frontend profile
   ↓
5. Frontend loads:
   - Call /api/session/validate with cookie
   - Backend validates session
   - Auto-refresh expired Discord tokens if needed
   - Return user data + guilds
   ↓
6. User is logged in
```

### Token Refresh Flow

```
Session validation → Check token_expires_at
                  ↓
         Token expired?
        /              \
      Yes              No
       ↓                ↓
Call refresh_token   Use existing
       ↓              access_token
Update database         ↓
       ↓             Return user data
Return user data
```

## Implementation Details

### Backend (Rust)

#### 1. Database Model (`bot/src/database/user_session.rs`)

```rust
pub struct Model {
    session_token: String,        // UUID, primary key
    user_id: String,              // Discord user ID
    discord_access_token: String, // For Discord API calls
    discord_refresh_token: String,// For token refresh
    token_expires_at: DateTime,   // When Discord token expires
    created_at: DateTime,         // Session creation
    last_used_at: DateTime,       // Last activity
}
```

#### 2. OAuth Handlers (`bot/src/api/oauth.rs`)

**oauth_callback**: 
- Exchanges authorization code for tokens
- Creates session with UUID token
- Stores in database
- Sets HTTP-only cookie
- Redirects to frontend

**validate_session**:
- Extracts session token from cookie
- Looks up session in database
- Checks token expiration
- Auto-refreshes if expired
- Updates last_used_at
- Returns user info + guilds

**logout**:
- Deletes session from database
- Clears cookie (Max-Age=0)

**refresh_access_token**:
- Calls Discord token refresh endpoint
- Returns new access + refresh tokens

#### 3. API Routes (`bot/src/api/server.rs`)

```rust
Router::new()
    .route("/api/oauth/login", get(oauth_login))
    .route("/api/oauth/callback", get(oauth_callback))
    .route("/api/session/validate", get(validate_session))
    .route("/api/session/logout", get(logout))
```

#### 4. State Management

Created `ApiState` struct to pass both config and database connection to handlers:

```rust
pub struct ApiState {
    pub config: Arc<Config>,
    pub db: DatabaseConnection,
}
```

### Frontend (Leptos/WASM)

#### 1. App Initialization (`website/kasuki-website/src/app.rs`)

```rust
create_effect(move |_| {
    wasm_bindgen_futures::spawn_local(async move {
        if let Ok(session_user) = validate_session().await {
            set_user.set(session_user);
        }
    });
});
```

#### 2. Session Validation

Uses Fetch API with credentials:

```rust
let mut opts = RequestInit::new();
opts.credentials(web_sys::RequestCredentials::Include);
```

This ensures cookies are sent with cross-origin requests.

#### 3. Logout (`website/kasuki-website/src/components/header.rs`)

```rust
let handle_logout = move |_| {
    wasm_bindgen_futures::spawn_local(async move {
        let _ = call_logout().await;  // Call backend endpoint
        set_user.set(None);            // Clear local state
    });
};
```

### Database

#### Migration (`migrations/create_user_session_table.sql`)

```sql
CREATE TABLE user_session (
    session_token VARCHAR(255) PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    discord_access_token TEXT NOT NULL,
    discord_refresh_token TEXT NOT NULL,
    token_expires_at TIMESTAMP NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP NOT NULL,
    INDEX idx_user_session_user_id (user_id),
    INDEX idx_user_session_last_used (last_used_at)
);
```

Indexes for:
- Fast user lookups
- Session cleanup queries

## Security Features

### 1. HTTP-Only Cookies
```
Set-Cookie: session_token=...; HttpOnly; SameSite=Lax
```
- Prevents JavaScript access (XSS protection)
- Browser automatically sends with requests

### 2. SameSite Protection
- `SameSite=Lax` prevents CSRF attacks
- Cookie only sent with same-site requests + top-level navigation

### 3. Secure Token Storage
- Session tokens are UUIDs (unpredictable)
- Separate from Discord tokens
- Database-backed (not in localStorage/sessionStorage)

### 4. Automatic Token Refresh
- Prevents expired token errors
- Seamless user experience
- No manual refresh needed

### 5. Activity Tracking
- `last_used_at` updated on each validation
- Enables session timeout policies
- Supports cleanup of stale sessions

## Configuration

Add to `config.toml`:

```toml
[api]
enabled = true
port = 8080

[api.oauth]
discord_client_id = "your_client_id"
discord_client_secret = "your_client_secret"
discord_redirect_uri = "http://localhost:8080/api/oauth/callback"
frontend_url = "http://localhost:8000"
```

## Setup Instructions

1. **Configure Discord OAuth:**
   - Go to Discord Developer Portal
   - Add redirect URI: `http://localhost:8080/api/oauth/callback`
   - Select scopes: `identify`, `guilds`, `email`
   - Copy Client ID and Secret to config

2. **Run Database Migration:**
   ```bash
   psql -U user -d kasuki -f migrations/create_user_session_table.sql
   ```

3. **Update Config:**
   - Add OAuth settings to `config.toml`
   - Set proper frontend_url for production

4. **Build and Deploy:**
   - Backend: `cargo build --release`
   - Frontend: Build Leptos app with KASUKI_API_URL env var

## Testing Checklist

- [ ] Login redirects to Discord
- [ ] Discord callback creates session
- [ ] Cookie is set in browser
- [ ] Page reload preserves login state
- [ ] Session validation returns user data
- [ ] Expired tokens are refreshed
- [ ] Logout clears session
- [ ] Multiple tabs maintain same session
- [ ] Session expires after inactivity

## Maintenance

### Session Cleanup

Run periodically to remove old sessions:

```sql
DELETE FROM user_session 
WHERE last_used_at < NOW() - INTERVAL '7 days';
```

Recommended: Set up cron job or scheduled task.

### Monitoring

Track in logs:
- Session creation rate
- Token refresh frequency
- Failed validation attempts
- Logout rate

## Future Enhancements

Potential improvements:
1. Redis caching for session lookups
2. Rate limiting on OAuth endpoints
3. Multi-device session management
4. Remember me / extended sessions
5. Session revocation API
6. Admin dashboard for session management

## Files Modified

### Backend (Rust)
- `bot/src/database/user_session.rs` (new)
- `bot/src/database/mod.rs`
- `bot/src/database/prelude.rs`
- `bot/src/api/oauth.rs`
- `bot/src/api/server.rs`
- `bot/src/api/mod.rs`
- `bot/src/main.rs`

### Frontend (Leptos/WASM)
- `website/kasuki-website/src/app.rs`
- `website/kasuki-website/src/components/header.rs`
- `website/kasuki-website/Cargo.toml`

### Database & Documentation
- `migrations/create_user_session_table.sql` (new)
- `migrations/README.md` (new)
- `API-DOCUMENTATION.md`

## Summary

This implementation provides a complete, secure session management solution for the Kasuki bot's web API. It follows industry best practices for OAuth authentication, token management, and session handling. The automatic token refresh ensures users stay logged in without manual intervention, while HTTP-only cookies and proper security headers protect against common web vulnerabilities.
