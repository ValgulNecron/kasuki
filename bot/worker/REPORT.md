# Worker Crate Report

## Overview

The `worker` crate is a standalone binary (`worker/src/main.rs`) that runs independently from the Discord bot. It connects to the same database and Discord HTTP API but has no runtime communication with the bot process. It executes three long-running background tasks on configurable intervals.

**Codebase size:** ~1,560 lines across 6 files.

---

## What It Does

### 1. Activity Management (`activity/anime_activity.rs` — 255 lines)

**Purpose:** Sends anime airing notifications to Discord guilds via webhooks.

**Flow:**
1. Every `activity_check` seconds (default: 1s), queries the `activity_data` table for rows whose `timestamp` matches the current second (`Utc::now().naive_utc()`).
2. For each matching row, spawns a tokio task that:
   - Optionally sleeps for `row.delay` seconds.
   - Decodes a base64-encoded image stored in the DB row.
   - Edits the webhook's avatar and name to match the anime.
   - Sends a localized embed with episode info via the webhook.
3. After sending, spawns another task to call `update_info`, which:
   - Fetches the anime's next airing episode from the AniList API.
   - If the anime has finished airing, deletes the activity row.
   - Otherwise, inserts a new row with the updated next-airing timestamp.

**Tables used:** `activity_data` (composite PK: `anime_id` + `server_id`).

---

### 2. Anisong Database Sync (`get_anisong_db.rs` — 465 lines)

**Purpose:** Populates the `anime_song` table with opening/ending song data from anisongdb.com.

**Flow:**
1. Every `anisong_update` seconds (default: 604,800 = 7 days), iterates through ANN IDs 1 to 100,000.
2. For each ID, spawns a tokio task (gated by a semaphore with limit **1**) that:
   - Waits on a governor rate limiter (20 req/s, burst 5).
   - POSTs to `https://anisongdb.com/api/annId_request` with the ANN ID.
   - On HTTP 429, retries with exponential backoff (up to 5 retries).
   - Parses the JSON response into `Vec<RawAniSongDB>`.
   - Filters entries that have an AniList ID.
   - Upserts each song into `anime_song` (composite PK: `anilist_id` + `ann_id` + `ann_song_id`).
3. After all 100k futures are collected, `join_all` awaits them and logs aggregate metrics.

**Tables used:** `anime_song`.

---

### 3. Random Statistics Update (`update_random_stats.rs` — 672 lines)

**Purpose:** Paginates through AniList's site statistics API to discover all anime/manga IDs, used by a "random anime/manga" feature.

**Flow:**
1. Every `random_stats_update` seconds (default: 86,400 = 1 day), loads the last-fetched page numbers from `random_stats` (row id=1, default: page 1796 for both).
2. Fetches anime stat pages in a loop:
   - Builds a GraphQL query (`AnimeStat`) for the current page.
   - If `has_next_page` is true, increments the page counter and continues.
   - On failure, retries up to 5 times with linear backoff (2s * attempt).
   - Sleeps 1s between successful pages to avoid rate limits.
3. Repeats the same loop for manga stat pages (`MangaStat`).
4. Upserts the final page numbers back into `random_stats`.

**Tables used:** `random_stats`.

---

### Startup & Lifecycle (`main.rs` — 143 lines)

1. Initializes tracing (INFO level).
2. Loads `config.toml`, connects to the database, creates a Discord `Http` client.
3. Spawns the three tasks as independent `tokio::spawn` handles.
4. Blocks on `Ctrl+C` signal, then exits (no graceful task cancellation).

### Dead Code (`structure.rs` — 26 lines)

Contains a `BotData` struct with fields like `shard_manager` and `user_blacklist` that are never used anywhere in the worker. This is leftover from a copy of the bot's struct.

---

## Issues and Improvement Opportunities

### Critical

#### 1. Activity timestamp matching is effectively broken
`send_activity` filters by `Timestamp.eq(now)` where `now` is `Utc::now().naive_utc()`. This compares against the **exact current second** (with sub-second precision depending on the DB type). Even with a 1-second polling interval, the chances of the stored timestamp matching the exact `NaiveDateTime` value are extremely slim due to sub-second drift. A range query (`timestamp <= now AND timestamp > last_check`) or a `timestamp <= now` with a "processed" flag would be far more reliable.

#### 2. No graceful shutdown for spawned tasks
`main.rs` catches `Ctrl+C` but doesn't propagate a cancellation signal to the three spawned tasks. The tasks use infinite loops with no `tokio::select!` on a shutdown receiver. This means in-flight DB writes or API calls are abruptly killed. The bot crate already uses a shutdown signal pattern — the worker should too.

#### 3. Anisong task spawns 100,000 futures then `join_all`
All 100k futures are collected into a `Vec` and awaited with `join_all`. Even though the semaphore limits concurrency to 1, all 100k `JoinHandle`s exist simultaneously in memory. This should use a streaming approach (e.g., `futures::stream::iter(...).buffer_unordered(N)`) or process in batches.

---

### High

#### 4. Semaphore concurrency is set to 1 — effectively sequential
The anisong semaphore is `Semaphore::new(1)`, meaning only one API request runs at a time. The comments say "10 was chosen as a balance" but the actual value is 1. Either raise it to a useful concurrency level (e.g., 5-10) or remove the semaphore/spawn overhead and just loop sequentially.

#### 5. `get_url` only supports PostgreSQL, panics on SQLite
`main.rs:get_url()` panics on any `db_type` other than `"postgresql"`. The bot crate supports SQLite as well. This is a duplicated, divergent version of URL construction — it should reuse the shared config's connection logic or at least support the same DB types.

#### 6. Redundant double-check in activity loop
After querying `WHERE timestamp = now`, the code iterates results and does `if now != row.timestamp { continue; }`. This second check is redundant since the DB already filtered it.

#### 7. Webhook avatar is re-uploaded on every notification
`send_specific_activity` decodes a base64 image, re-encodes it as a PNG attachment, and calls `webhook.edit()` to set the avatar **every time** a notification fires. This is expensive (image decode + Discord API call) and unnecessary if the avatar hasn't changed. The avatar could be stored as a URL or cached.

---

### Medium

#### 8. No SQLite support
The `get_url` function only handles `"postgresql"` and panics otherwise. The shared config and bot support SQLite. The worker should too.

#### 9. `structure.rs` is dead code
The `BotData` struct in `structure.rs` is never used. It references `ShardManager` which doesn't apply to the worker. Should be deleted.

#### 10. Verbose logging — excessive trace/debug noise
`update_random_stats.rs` is 672 lines, but roughly 40% of it is logging statements. Many trace calls log obvious information like `"Returning true to indicate more pages available"` or `"Exiting update_random function"`. This makes the core logic harder to read. The tracing instrumentation macros already handle entry/exit logging.

#### 11. `update_page` always reads `manga.page_info` regardless of type
When `update_anime=true`, the code fetches `AnimeStat` but then navigates `data.site_statistics.manga.page_info` to check `has_next_page`. This works only because `AnimeStat` and `MangaStat` share the same GraphQL response shape mapped to the same `AnimeStat` type, but it's confusing and fragile.

#### 12. Duplicated anime/manga update logic
The anime and manga update loops in `update_random` are nearly identical (~80 lines each). They differ only in which page counter is updated and which boolean flag is passed. This should be a single function parameterized by category.

#### 13. Media URLs are constructed even when empty
When `hq`, `mq`, or `audio` are `None`, the code produces URLs like `"https://files.catbox.moe/"` (empty filename). These broken URLs get stored in the database. Should store an empty string or `None` instead.

#### 14. No connection pooling configuration
`sea_orm::Database::connect(url)` is called with defaults. For a long-running worker making frequent queries, configuring pool size, idle timeout, and max lifetime would improve reliability.

---

### Low / Housekeeping

#### 15. Unused dependencies in Cargo.toml
`image`, `palette`, `rayon`, `uuid`, `dashmap`, `async-trait` (not listed but implied by shared) are declared but appear unused in the worker's source files. Removing them would speed up compilation.

#### 16. No health check or observability endpoint
The worker runs as a headless process. There's no way to check if it's alive, what cycle it's on, or when the last successful run was. A simple HTTP health endpoint or metrics export (e.g., Prometheus) would help with monitoring.

#### 17. Hardcoded ANN ID range (1 to 100,000)
The upper bound of 100k is a magic number. As the ANN database grows, this will need to increase. It should be configurable or, better, use the API's pagination rather than brute-forcing ID ranges.

#### 18. No incremental anisong updates
Every cycle re-fetches all 100k ANN IDs. There's no mechanism to track which IDs have already been processed or to only fetch new/updated entries. An incremental approach (e.g., tracking the last-fetched ID or using API timestamps) would drastically reduce the 7-day cycle time and API load.

#### 19. Error in activity update — insert without conflict handling
`update_info` calls `ActivityData::insert(new_activity).exec()` without `.on_conflict()`. If the row already exists (same `anime_id` + `server_id`), this will fail with a unique constraint violation instead of updating.

#### 20. `base64` + `image` crate imported but image processing is minimal
The activity task decodes base64 to raw bytes and passes them to Serenity's `CreateAttachment`. The `image` crate is in Cargo.toml but isn't actually used for any processing. The base64 decode could use the `base64` crate's simpler `STANDARD.decode()` API instead of the streaming `DecoderReader`.

---

## Summary of Recommended Changes (Priority Order)

| Priority | Change | Impact |
|----------|--------|--------|
| P0 | Fix activity timestamp matching (range query or `<=` with flag) | Notifications likely never fire correctly |
| P0 | Add graceful shutdown with cancellation tokens | Prevents data corruption on stop |
| P1 | Stream anisong futures instead of `join_all` on 100k handles | Memory efficiency |
| P1 | Raise semaphore to >1 or simplify to sequential loop | Either get real concurrency or remove overhead |
| P1 | Support SQLite in `get_url` or reuse shared DB connect | Parity with bot crate |
| P1 | Fix `update_info` insert to handle conflicts (upsert) | Prevents runtime errors |
| P2 | Deduplicate anime/manga update loops | Maintainability |
| P2 | Fix empty media URL construction | Data quality |
| P2 | Remove dead `structure.rs` | Cleanliness |
| P2 | Reduce log verbosity in `update_random_stats.rs` | Readability |
| P3 | Make ANN ID range configurable | Flexibility |
| P3 | Add incremental anisong updates | Performance |
| P3 | Add health check endpoint | Observability |
| P3 | Remove unused Cargo dependencies | Build time |
