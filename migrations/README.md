# Database Migrations

This directory contains SQL migration files for the Kasuki database.

## Running Migrations

### PostgreSQL

To apply the user_session table migration, run the following SQL against your PostgreSQL database:

```bash
psql -U your_user -d kasuki -f migrations/create_user_session_table.sql
```

Or connect to your database and execute:

```sql
\i migrations/create_user_session_table.sql
```

### SQLite

For SQLite databases, the table creation needs a slight modification since SQLite doesn't support all PostgreSQL features:

```sql
CREATE TABLE IF NOT EXISTS user_session (
    session_token TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    discord_access_token TEXT NOT NULL,
    discord_refresh_token TEXT NOT NULL,
    token_expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_used_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_session_user_id ON user_session(user_id);
CREATE INDEX idx_user_session_last_used ON user_session(last_used_at);
```

## Migrations Included

### create_user_session_table.sql

Creates the `user_session` table for OAuth session management. This table stores:
- Session tokens (UUID strings)
- User IDs from Discord
- Discord OAuth access and refresh tokens
- Token expiration timestamps
- Session creation and last-used timestamps

This is required for the web API authentication to work properly.

## Session Cleanup

To clean up old/expired sessions, you can run a periodic cleanup query:

```sql
-- Delete sessions not used in the last 7 days
DELETE FROM user_session WHERE last_used_at < NOW() - INTERVAL '7 days';
```

Or for SQLite:

```sql
DELETE FROM user_session WHERE last_used_at < datetime('now', '-7 days');
```

Consider setting up a cron job or scheduled task to run this periodically.
