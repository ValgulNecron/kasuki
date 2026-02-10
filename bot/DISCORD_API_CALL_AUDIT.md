# Discord API Call Audit - Kasuki Bot

## CRITICAL (Immediate Attention)

### 1. Guild Member Pagination Loop
**File:** `bot/src/server_image/calculate_user_color.rs:321-377`
**Function:** `get_member()`

Tries cache first via `guild.to_guild_cached(&ctx.cache)`, but falls back to a pagination loop calling `guild.members(&ctx.http, Some(1000), ...)` repeatedly until all members are fetched. A guild with 10,000 members triggers ~10 paginated API calls.

Called from `color_management()` which runs for **every guild in cache** during `ready()`, guild creation, and member addition events.

---

### 2. All-Guild Color Management Loop
**File:** `bot/src/server_image/calculate_user_color.rs:275-319`
**Function:** `color_management()`

Iterates every guild in cache and spawns concurrent member fetch tasks (in chunks of 4 via `FuturesUnordered`). If bot is in 100 guilds, this triggers 100+ member fetch operations, each potentially making multiple paginated API calls.

**Triggered by:** `ready()` event (every startup), guild creation, member addition.

---

### 3. Per-Member Profile Picture Downloads
**File:** `bot/src/server_image/calculate_user_color.rs:52-120`
**Function:** `calculate_users_color()`

For each member: downloads profile picture via HTTP, does a DB lookup, calculates color, and does a DB insert. For 5,000 members this means 5,000 HTTP downloads + 5,000 DB queries sequentially.

---

### 4. All-Guild Image Generation Loop
**File:** `bot/src/server_image/generate_server_image.rs:261-298`
**Function:** `server_image_management()`

Loops through ALL guilds in cache. Local image generation is spawned as a tokio task (non-blocking), but global image generation is a **blocking await** per guild.

**Triggered by:** `ready()` event, guild creation, every 100 member additions.

---

### 5. Guild Partial Fetch Loop on Startup
**File:** `bot/src/event_handler.rs:576-589`
**Function:** `ready()`

During startup, fetches partial guild info for every guild:
```rust
for guild in ctx.cache.guilds() {
    let partial_guild = guild.to_partial_guild(&ctx.http).await;  // API call per guild
    ctx.chunk_guild(partial_guild.id, None, true, ...);
}
```
100 guilds = 100 API calls on every restart.

---

## HIGH (Should Fix)

### 6. Guild Member Addition Triggers
**File:** `bot/src/event_handler.rs:412-453`
**Function:** `guild_member_addition()`

Each new member triggers:
- `get_specific_user_color()` (profile picture download + DB ops)
- `add_user_data_to_db()` (DB operation)
- Every 100 members: `server_image_management()` (all-guild image regeneration)

Note: No longer calls `to_user()` API — uses `member.user.clone()` directly.

---

### 7. Member Iterator with N+1 DB Queries
**File:** `bot/src/command/anilist_server/list_register_user.rs:133-162`
**Function:** `get_the_list()`

Uses `guild.id.members_iter(&ctx.http)` pagination + 1 DB query per member to check registration. For 5,000 members = ~5 pagination API calls + 5,000 DB queries.

---

### 8. Anime Activity Webhook Operations
**File:** `worker/src/activity/anime_activity.rs:72-154`

Background task iterating activities. Each activity spawns a tokio task making 3 API calls:
1. `Webhook::from_url()` — fetch webhook
2. `webhook.edit()` — edit webhook
3. `webhook.execute()` — send message

For 50 concurrent activities = 150 API calls per check interval.

---

## MEDIUM (Consider Fixing)

### 9. Command Registration Loop
**File:** `bot/src/register/registration_dispatcher.rs:76-133`

On startup (when `remove_old_commands` is true), deletes all existing commands one-by-one then recreates them. 50 commands x 100 guilds = potentially 10,000+ API calls on every restart. Should use bulk `set_global_commands()` / `set_guild_commands()` instead.

---

### 10. Random Stats Update Pagination
**File:** `worker/src/update_random_stats.rs:298-445`

Background task paginating AniList API. Has 1-second delays between calls and max failure limits (5 retries), so impact is managed.

---

### 11. Pixel-by-Pixel Thread Spawning
**File:** `bot/src/server_image/generate_server_image.rs:115-162`

Not an API call issue, but spawns a thread per pixel (128x128 = 16,384 threads, batched by `THREAD_POOL_SIZE`) for image generation. High CPU cost, especially when triggered for every guild.

---

## Previously Fixed

### ~~Per-Member API Calls in Guild Members Chunk~~ (Fixed)
**File:** `bot/src/event_handler.rs:469-470`

Previously called `member.user.id.to_user(&ctx.http).await` for each member in a chunk (10,000+ API calls during startup). Now uses `member.user.clone()` directly.

### ~~Presence Update Per-User API Call~~ (Fixed)
**File:** `bot/src/event_handler.rs:520`

Previously called `new_data.user.id.to_user(&ctx).await` (HTTP API call) on every presence change. Now uses `new_data.user.to_user()` which is a local conversion with no API call.

### ~~Guild Member Addition to_user() Call~~ (Fixed)
**File:** `bot/src/event_handler.rs:444`

Previously called `member.user.id.to_user(&ctx.http).await`. Now uses `member.user.clone()` directly.

---

## Recommendations

| Issue | Recommendation |
|-------|---------------|
| Member pagination loops | Cache member lists, only re-fetch on invalidation |
| All-guild loops | Process only changed guilds, not all guilds every time |
| Profile picture downloads | Cache images with TTL, skip unchanged avatars |
| Startup guild fetch | Use cache data or lazy-load guild info |
| Command registration | Use bulk `set_global_commands()` / `set_guild_commands()` instead of per-command calls |
| N+1 DB queries | Batch query all registered users for the guild in one query |
| Image generation | Generate on-demand (when requested) instead of eagerly on every event |
| Webhook operations | Batch webhook executions, cache webhook objects |
