# API Server Report

## Overview

The `api-server` crate is a standalone Axum-based HTTP server that provides Discord OAuth2 authentication and a minimal user profile endpoint. It runs independently from the main bot process and shares only the `shared` crate for configuration loading.

**Total source files:** 5 (`main.rs`, `api/mod.rs`, `api/auth.rs`, `api/oauth.rs`, `api/server.rs`)
**Total lines of code:** ~420
**Endpoints:** 4

---

## What It Does

### Startup Flow

1. `main.rs` initializes tracing, loads `config.toml` via `shared::config::Config::new()`, wraps it in `Arc`, and calls `start_api_server()`.
2. `api/mod.rs` checks `config.api.enabled` — if disabled, the server exits silently.
3. `api/server.rs` builds the Axum router, binds to `0.0.0.0:{port}`, and starts serving.

### Endpoints

| Method | Route                 | Auth | Purpose                                                                                                                                                      |
|--------|-----------------------|------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `GET`  | `/api/health`         | None | Returns `{"status": "ok", "service": "kasuki-api"}`                                                                                                          |
| `GET`  | `/api/oauth/login`    | None | Redirects to Discord's OAuth2 authorization page                                                                                                             |
| `GET`  | `/api/oauth/callback` | None | Handles Discord's OAuth2 callback, exchanges code for token, fetches user info + guilds, generates a JWT, redirects to frontend with the JWT in the URL hash |
| `GET`  | `/api/user/me`        | JWT  | Returns the authenticated user's profile and guild list from an in-memory cache                                                                              |

### Authentication Flow

1. User visits `/api/oauth/login` → redirected to Discord with `identify guilds email` scopes.
2. Discord redirects back to `/api/oauth/callback?code=...`.
3. Server exchanges the authorization code for an access token via `POST https://discord.com/api/v10/oauth2/token`.
4. Server fetches `GET /users/@me` and `GET /users/@me/guilds` from Discord using the access token.
5. User info and guilds are stored in a global `DashMap` keyed by user ID.
6. A JWT (HS256, 24-hour expiry) is generated containing `sub` (user ID), `username`, and `exp`.
7. User is redirected to `{frontend_url}/#/profile?jwt={token}`.
8. Frontend stores the JWT and sends it as `Authorization: Bearer {token}` on subsequent requests.

### Data Storage

The server is **fully stateless** with respect to persistent storage. It uses:
- A global `static USER_CACHE: LazyLock<DashMap<String, (UserInfo, Vec<Guild>)>>` for in-memory user data.
- No database connections. No SeaORM. No Redis.

### Middleware

- **CORS:** `allow_origin(Any)`, `allow_methods(Any)`, `allow_headers(Any)` — applied globally.
- **Auth:** JWT validation middleware applied only to `/api/user/*` routes. Extracts `Bearer` token from the `Authorization` header, base64-decodes the JWT secret from config, validates with HS256, and inserts `Claims` into request extensions.

---

## Issues and Improvement Opportunities

### Critical

#### 1. In-Memory Cache Has No TTL and No Eviction
The `USER_CACHE` `DashMap` grows unboundedly. Every user who logs in stays cached forever (until server restart). There is no TTL, no LRU eviction, and no maximum size.

**Impact:** Memory leak proportional to the number of unique logins over the server's lifetime.

**Fix:** Replace `DashMap` with a TTL-aware cache like `moka` (already used in the bot crate). Set a TTL matching the JWT expiry (24 hours) and a maximum capacity.

#### 2. User Data Goes Stale Immediately
User info and guilds are fetched once at login and never refreshed. If a user changes their username/avatar or joins/leaves guilds, the cached data is permanently stale until they log in again.

**Fix:** Either (a) re-fetch from Discord on each `/api/user/me` request (with short caching), (b) store the Discord access token and refresh token to re-fetch on demand, or (c) set a short cache TTL and force re-authentication.

#### 3. `.unwrap()` on Base64 Decode of JWT Secret
In `auth.rs:53` and `oauth.rs:182`, the JWT secret is base64-decoded with `.unwrap()`. If the config value is invalid base64, the server panics.

**Fix:** Return a `500 Internal Server Error` instead of panicking. Validate the secret at startup.

#### 4. Discord Access Token and Refresh Token Are Discarded
After the OAuth callback, the `TokenResponse` (containing `access_token` and `refresh_token`) is thrown away. This means there is no way to refresh user data or perform actions on behalf of the user later.

**Fix:** If future features need to act on behalf of users, store the tokens (encrypted) alongside the user cache entry, with proper expiry tracking.

### Security

#### 5. CORS Allows Any Origin
`CorsLayer::new().allow_origin(Any)` permits any website to make authenticated requests to the API. This is suitable for local development but dangerous in production — it enables CSRF-like attacks if the frontend stores the JWT in a cookie or if browser extensions relay it.

**Fix:** Restrict `allow_origin` to the configured `frontend_url` in production. Consider making it configurable.

#### 6. JWT Passed in URL Hash Fragment
The JWT is passed via `{frontend_url}/#/profile?jwt={token}`. While the hash fragment is not sent to the server in HTTP requests, it can leak through browser history, referrer headers (if the fragment is accidentally included), and JavaScript access. It is also visible in the browser address bar.

**Fix:** Consider using an HTTP-only secure cookie for the JWT, or use a short-lived authorization code that the frontend exchanges for a JWT via a separate POST endpoint (similar to the PKCE flow).

#### 7. No CSRF Protection on OAuth Flow
The OAuth login does not include a `state` parameter. This makes the flow vulnerable to CSRF attacks where an attacker tricks a user into completing an OAuth flow with the attacker's account.

**Fix:** Generate a random `state` value, store it server-side (or in a signed cookie), pass it to Discord, and verify it in the callback.

#### 8. No Rate Limiting
There is no rate limiting on any endpoint. The OAuth endpoints are particularly sensitive since they make outbound requests to Discord's API.

**Fix:** Add rate limiting middleware (e.g., `tower::limit` or `governor`) at minimum on `/api/oauth/login` and `/api/oauth/callback`.

### Architecture

#### 9. No Database Integration
The API server has zero database access despite the workspace having a full SeaORM setup with user tables. This means:
- User profile data cannot survive a server restart.
- The API cannot serve bot-related data (user settings, anime lists, server configurations).
- There is no persistent session management.

**Fix:** Add a database connection (reuse the `shared` crate's database setup). Persist user sessions and use the database as the source of truth rather than an in-memory cache.

#### 10. Only One Protected Endpoint
The server authenticates users but then offers only a single endpoint (`/api/user/me`) that returns the same data that was cached at login time. There is essentially no functionality beyond "log in and see your cached Discord profile."

**Fix:** This depends on the project's goals. Potential endpoints:
- `GET /api/user/settings` — user's bot preferences
- `GET /api/guilds/:id/config` — guild-specific bot configuration
- `PUT /api/guilds/:id/config` — update guild bot settings (dashboard)
- `GET /api/user/anilist` — user's linked AniList data
- `GET /api/stats` — bot statistics

#### 11. `reqwest::Client` Created Per Request
In `exchange_code_for_token()`, `get_user_info()`, and `get_user_guilds()`, a new `reqwest::Client` is constructed for every call. `reqwest::Client` manages a connection pool internally, so recreating it per request wastes that benefit.

**Fix:** Create a single `reqwest::Client` at startup and share it via Axum state (alongside `Arc<Config>`).

#### 12. Duplicate Type Definitions
`server.rs` defines its own `User` and `Guild` structs that are subsets of the `oauth::UserInfo` and `oauth::Guild` types. This creates unnecessary mapping code and duplication.

**Fix:** Either (a) use `#[serde(skip_serializing)]` on fields you don't want in the response, or (b) consolidate into a single set of types with serde attributes controlling serialization.

#### 13. Error Type Inconsistency
The codebase uses three different error strategies:
- `ApiError` enum in `auth.rs` (implements `IntoResponse`) — but is never actually used outside its own file.
- Raw `StatusCode` returns from the auth middleware.
- `Box<dyn std::error::Error>` in the OAuth helper functions.
- String-based error redirects in the callback handler.

**Fix:** Adopt a unified error type (e.g., a single `AppError` enum) that implements `IntoResponse` and covers all cases. Use `thiserror` for the error definitions.

### Code Quality

#### 14. `ApiError` Enum Is Dead Code
`ApiError::Forbidden` and `ApiError::InternalServerError` are defined but never used. The `#[allow(dead_code)]` annotation suppresses the warning, but the enum itself is never constructed anywhere.

**Fix:** Either use it consistently in handlers or remove it.

#### 15. `health_check()` Is in `oauth.rs`
The health check endpoint has nothing to do with OAuth. It is placed in `oauth.rs` for convenience rather than correctness.

**Fix:** Move it to `server.rs` or a dedicated `health.rs` module.

#### 16. No Tests
There are zero tests — no unit tests, no integration tests, no handler tests.

**Fix:** Add at least:
- Unit tests for JWT encoding/decoding.
- Integration tests for the health endpoint.
- Mock-based tests for the OAuth flow using `wiremock` or `mockito`.

#### 17. No Graceful Shutdown
The server has no graceful shutdown signal handling. `axum::serve(listener, app).await` blocks forever. There is no way to shut down cleanly (drain connections, flush logs).

**Fix:** Use `axum::serve(...).with_graceful_shutdown(shutdown_signal())` with a `tokio::signal::ctrl_c()` handler.

#### 18. No Request Logging / Tracing Middleware
Individual handlers log at various levels, but there is no request-level tracing middleware. This makes it difficult to correlate logs for a single request or track request latency.

**Fix:** Add `tower_http::trace::TraceLayer` to the router for automatic request/response logging with span-based correlation.

---

## Summary

The API server is a minimal OAuth2 login gateway. It authenticates users via Discord, caches their profile in memory, and issues JWTs. It currently serves as a foundation but provides very little functionality beyond login.

**The most impactful improvements, roughly in priority order:**

1. Add `state` parameter to the OAuth flow (security).
2. Replace the unbounded `DashMap` cache with `moka` + TTL (reliability).
3. Fix `.unwrap()` calls on base64 decoding (reliability).
4. Restrict CORS to the configured frontend URL (security).
5. Add database integration for persistent user data and bot settings (functionality).
6. Add more endpoints to make the API useful for a dashboard (functionality).
7. Reuse a single `reqwest::Client` (performance).
8. Add `TraceLayer` for request logging (observability).
9. Add rate limiting (security).
10. Add tests (maintainability).
