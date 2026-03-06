# Memory Usage Report

Analysis of the Kasuki bot codebase for memory-heavy patterns, potential leaks, and optimization opportunities.

**Last updated:** 2026-03-04

---

## Critical Issues

### ~~1. Memory Leak via `Box::leak` in `load_localization`~~ (FIXED)

**Status:** Resolved. The `Box::leak` call and the `load_localization` function have been removed from `shared/src/localization.rs`. All localization now uses the Fluent `USABLE_LOCALES` static loader.

---

### ~~2. `UserColor::find().all()` Loads Entire Table into RAM~~ (FIXED)

**Status:** Resolved.
- `info.rs` now uses `UserColor::find().count()` (SQL `COUNT(*)`) instead of loading all rows
- `enqueue_local_server_image` no longer queries DB — sends only `(user_id, profile_picture_url)` from Discord API; worker fetches color data from DB
- `enqueue_global_server_image` sends empty `members` vec — worker fetches all `UserColor` records from DB directly in a single query
- Removed `cached_color` from `MemberColorData` (redundant — worker always loads from DB)
- Bot process no longer loads the `user_color` table at all for server image tasks

---

### 3. AniList/VNDB Caches: Partially Improved, Remaining Issues

**File:** `shared/src/cache.rs`, `shared/src/config.rs`

**What's been fixed (2025-11):**
- `CacheConfig` now has configurable fields: `cache_type` (memory/redis), `max_capacity`, `ttl_secs`, `host`, `port`, `password`
- Redis backend added as alternative to Moka (in-memory), selectable via `config.toml`
- `CacheInterface::from_config()` respects configuration settings
- Graceful fallback: if Redis connection fails, falls back to in-memory cache automatically

**What's still open:**
- Cache keys are still **full GraphQL query strings** concatenated with serialized variables (hundreds of bytes per key), wasting memory on key storage
- Default capacity is still 10,000 entries — 3 instances (bot anilist, bot vndb, worker anilist) = up to **150-300 MB total** at max capacity
- Caches are still double-wrapped: `BotData` wraps `CacheInterface` in `Arc<RwLock<CacheInterface>>`. Moka is already `Clone + Send + Sync` internally; the `RwLock` adds unnecessary contention on concurrent reads (~30+ call sites pass `Arc<RwLock<CacheInterface>>`)

**Remaining fix:**
- Hash the query+variables into a fixed-size key (e.g., `u64` via `xxhash` or `seahash`)
- Reduce default capacity to 1,000-2,000 (more realistic for a Discord bot)
- Remove the `Arc<RwLock<>>` wrapper — Moka handles concurrency internally

---

## High-Severity Issues

### 4. `reqwest::Client::new()` Created Per Request

Creating a new HTTP client per request discards the connection pool and forces a fresh TLS handshake every time.

`BotData` already has a shared `http_client: Arc<Client>` field (initialized in `main.rs`), but the shared library code does not use it.

| File | Line | Context |
|---|---|---|
| `shared/src/anilist/make_request.rs` | 104 | Every AniList API call via the shared library |
| `shared/src/vndb/common.rs` | 20, 61 | Every VNDB API call (both `do_request` and `do_request_with_json`) |

**Note:** `bot/src/command/ai/image.rs` now correctly uses `bot_data.http_client.clone()` (line 59) for its main request. However, `image.rs:290` still creates `Client::new()` inside `get_image_from_response()` for downloading images in a loop.

**Fix:** Pass the shared `reqwest::Client` from `BotData` through to the shared library's request functions instead of creating new clients.

### 5. Guild Member Chunking Loads All Members of All Guilds

**File:** `bot/src/handlers/ready.rs:57-63`

```rust
for guild in ctx.cache.guilds() {
    ctx.chunk_guild(guild, None, true, ChunkGuildFilter::None, None);
}
```

This requests **all members** for every guild from Discord, populating the serenity cache with full `Member` structs (user data, roles, nickname, etc.) for every user in every server. No `CacheSettings` are configured anywhere -- the cache grows unboundedly.

For a bot in 100 guilds averaging 500 members each: **50,000 Member structs** cached in memory.

**Fix:**
- Only chunk guilds when member data is actually needed (lazy chunking)
- Configure `CacheSettings` with `max_messages` and consider disabling member caching if not essential
- If member caching is needed, use `ChunkGuildFilter` to request only specific members

### 6. `user_blacklist.read().await.clone()` Copies Entire Vec

**File:** `bot/src/server_image/generate_server_image.rs:46, 105`

```rust
let user_blacklist = bot_data.user_blacklist.read().await.clone();
```

The `user_blacklist` field in `BotData` is `Arc<RwLock<Vec<String>>>`. Both `enqueue_local_server_image` and `enqueue_global_server_image` clone the entire `Vec<String>` out of the RwLock. The clone is serialized into the Redis task payload.

**Fix:** Switch to an `Arc<HashSet<String>>` that can be cheaply cloned via Arc. The serialization into the task payload is necessary, but holding the read guard while serializing (instead of cloning first) would avoid the extra allocation.

### 7. `lava_client.read().await.clone()` in Every Music Command

**Files:** Music command files (`play.rs`, `skip.rs`, `pause.rs`, and others)

```rust
let lava_client = bot_data.lavalink.read().await.clone();
```

The `lavalink` field is `Arc<RwLock<Option<LavalinkClient>>>`. Clones the inner `Option<LavalinkClient>` on every music command invocation. If `LavalinkClient` is not just an Arc wrapper, this duplicates internal state.

**Fix:** Hold the read guard for the duration of the operation, or ensure the inner type is `Arc`-wrapped so cloning is a pointer bump.

---

## Medium-Severity Issues

### 8. Server Image Generation: 64 MB Canvas + Bulk Member Clone

**File:** `image_generation/src/mosaic.rs` (worker)

- A `DynamicImage::new_rgba8(4096, 4096)` allocates **64 MB** of RGBA pixel data
- Per-user avatar images (128x128, ~65 KB each) are decoded and held simultaneously

The 64 MB canvas allocation and image compositing happen in the dedicated `image_generation` worker process, not the bot. The bot's memory footprint during server image operations is limited to the member list + task serialization.

**Fix:**
- Process members in batches rather than cloning all at once
- Consider a smaller canvas (2048x2048 = 16 MB) unless the resolution is essential
- Stream and composite avatar tiles incrementally

### 9. Steam Game HashMap (~7-10 MB, 3x Peak During Refresh)

**File:** `bot/src/structure/steam_game_id_struct.rs`

The Steam API returns ~170,000 apps. During refresh, the raw JSON body, the parsed `Vec<App>`, and the new `HashMap` all exist simultaneously before the old map is dropped. Peak memory is ~3x the steady-state ~7-10 MB. Uses `HashMap<String, u128>` when Steam app IDs fit in `u32`.

**Fix:**
- Parse directly from the response stream into the HashMap (avoid intermediate `Vec<App>`)
- Consider using `u32` instead of `u128` for app IDs (Steam app IDs fit in u32)
- Use `CompactString` or intern game names if memory is critical

### 10. `push_str(format!(...).as_str())` -- Unnecessary Temp String

**Files:**
- `bot/src/structure/run/anilist/character.rs:105, 115, 125`
- `bot/src/structure/run/anilist/media.rs:394, 433, 462, 465, 467`

```rust
staff_text.push_str(format!("{}: {}\n", staff_name, role).as_str());
```

Each `format!()` allocates a temporary `String` on the heap, only to immediately borrow it.

**Fix:** Use `write!(staff_text, "{}: {}\n", staff_name, role).unwrap()` from `std::fmt::Write` to write directly into the target String with zero intermediate allocation.

### 11. Double `collect()` + Struct Clones in `get_tag_list` / `get_genre_list`

**File:** `bot/src/structure/run/anilist/user.rs:327-346`

```rust
let vec = vec.iter()
    .map(|tag| tag.clone().unwrap().tag.clone().unwrap().name.clone())
    .collect::<Vec<_>>();
let vec = vec.into_iter().take(5).collect::<Vec<_>>();
vec.join("/")
```

Three levels of `.clone()` per element, two intermediate `Vec` allocations.

**Fix:**
```rust
vec.iter()
    .filter_map(|tag| Some(tag.as_ref()?.tag.as_ref()?.name.clone()))
    .take(5)
    .collect::<Vec<_>>()
    .join("/")
```

Same pattern in `compare.rs`.

### 12. `vn.image.clone()` Called Twice Then Moved

**Files:** `bot/src/command/vn/game.rs:142-150`, `bot/src/command/vn/character.rs:196-213`

```rust
let sexual = match vn.image.clone() { ... };   // clone 1
let violence = match vn.image.clone() { ... }; // clone 2
let url = match vn.image { ... };              // move
```

**Fix:** Single destructure:
```rust
let (sexual, violence, url) = match vn.image {
    Some(image) => (image.sexual, image.violence, Some(image.url)),
    None => (2.0, 2.0, None),
};
```

### 13. Database Tables With Unbounded Growth

| Table | File | Issue |
|---|---|---|
| `command_usage` | `bot_data.rs:73-88` | Inserts a row per command invocation, never pruned |
| `ping_history` | `launch_task/ping_manager.rs` | Inserts per shard per tick (~5,760 rows/day with 4 shards) |
| `vocal` | Voice session handler | Sessions inserted on leave, never cleaned |

No cleanup tasks have been added. The existing launch tasks (`game_management`, `ping_manager`, `user_blacklist`, `bot_info_update`, `queue_publisher`) contain no pruning logic.

**Fix:** Add a periodic cleanup task that deletes rows older than N days, or use partitioned tables with automatic drop of old partitions.

### 14. Rate Limiter `DashMapStateStore` Has No Eviction

**File:** `api-server/src/api/rate_limit.rs`

The `governor` rate limiter uses a `DashMapStateStore<String>` keyed by IP address. There is no eviction strategy -- the map grows unboundedly as unique IPs make requests over the server's lifetime.

**Fix:** Use `governor`'s `keyed::DefaultKeyedRateLimiter` with periodic cleanup, or switch to a time-bounded store.

---

## Low-Severity Issues

### 15. API Server Cache Capacities Oversized

**File:** `api-server/src/api/state.rs`

| Cache | Capacity | TTL | Realistic Need |
|---|---|---|---|
| `user_cache` | 10,000 | 24h | 100-500 |
| `auth_codes` | 10,000 | 5m | 100-200 |
| `oauth_states` | 10,000 | 10m | 100-200 |

The user cache stores `(UserInfo, Vec<Guild>)` per user. A Discord power user may belong to 100+ guilds. At 10,000 entries with 24h TTL, this is excessive for a bot's API server.

### 16. `DEFAULT_STRING: &String` with `.clone()` Callers

**File:** `bot/src/constant.rs:52`

```rust
pub const DEFAULT_STRING: &String = &String::new();
```

Callers use `DEFAULT_STRING.clone()` which allocates a new heap String for a fallback empty value.

**Fix:** Use `&str = ""` and `.unwrap_or_default()` on `Option<String>`.

### 17. `if x != String::new()` Allocates for Comparison

**Files:** `bot/src/command/anilist_user/staff.rs:139,150`, `bot/src/structure/run/anilist/media.rs:461,464,466`

**Fix:** Use `!x.is_empty()`.

### 18. `anyhow!(format!(...))` Double Allocation

**Files:** `bot/src/command/admin/anilist/add_activity.rs:209`, `bot/src/command/admin/anilist/delete_activity.rs:59,120`, `bot/src/command/ai/image.rs:274`

**Fix:** `anyhow!("message {}", var)` accepts format args directly.

### 19. `Vec::from(body.clone())` on Bytes

**File:** `bot/src/command/ai/image.rs:301`

`Bytes::clone()` is cheap (Arc bump), but `Vec::from()` copies the entire buffer. The `.clone()` before `Vec::from()` is redundant.

### 20. Minigame Inventory: Cloning Vec of DB Structs to Sort

**Files:** `bot/src/command/minigame/fish_inventory.rs`, `bot/src/command/minigame/inventory.rs`

```rust
let mut sorted_fish = fish_list.clone();
sorted_fish.sort_by(...)
```

Clones a `Vec<(UserInventoryModel, ItemModel)>` (full DB row structs) just to sort.

**Fix:** Sort a `Vec<usize>` of indices, or take ownership of the data instead of borrowing and cloning.

### 21. Unused Clone

**File:** `bot/src/command/anilist_server/list_all_activity.rs:64`

```rust
let _config = bot_data.config.clone(); // never used
```

### 22. Redundant `.clone()` at End of Builder Chain

**Files:** `bot/src/command/ai/image.rs:160-162`, `bot/src/command/anilist_user/seiyuu.rs:227`

```rust
EmbedsContents::new(...).add_files(command_files).clone(); // clone of owned value
```

---

## Architecture Change (2025-02): Image Generation Worker

Server image generation and user color calculation have been moved from the bot process into a dedicated `image_generation` worker binary. Communication happens via a Redis queue.

**What moved out of the bot:**
- Color calculation (CIELAB Delta-E 2000 matching)
- Mosaic generation (4096x4096 canvas compositing via rayon)
- Image saving (local filesystem or catbox upload)
- DB upserts for `user_color` and `server_image` tables

**What stays in the bot:**
- Guild member fetching from Discord API/cache
- Pre-fetching `UserColor` records to build task payloads
- Publishing serialized `ImageTask` to Redis

**Impact on memory issues:**
- Issues #2 and #8 are **mitigated** for the bot process -- the heavy allocations now happen in the worker
- Issue #2 still applies to the bot's `enqueue_local_server_image` and `enqueue_global_server_image` (loads full color table for payload)
- The worker no longer bulk-loads `UserColor` — it fetches per-user via `.one()` with filter
- The bot gains a small Redis connection overhead (~few KB)

---

## Summary by Estimated Memory Impact

| Priority | Issue | Status | Estimated Impact (bot process) |
|---|---|---|---|
| ~~Critical~~ | ~~#1 `Box::leak` in `load_localization`~~ | **FIXED** | ~~Unbounded growth (leak)~~ |
| ~~Critical~~ | ~~#2 `UserColor::find().all()` in bot (payload + info command)~~ | **FIXED** | ~~Up to several GB~~ |
| Critical | #3 AniList/VNDB caches (keys unhashed, double-wrapped) | **Partial** | 150-300 MB |
| High | #5 Guild member chunking (all members, all guilds) | Open | Hundreds of MB for large bots |
| High | #4 `reqwest::Client::new()` per request (shared lib) | Open | Connection pool churn (indirect) |
| Medium | #8 Server image 64 MB canvas + avatars | Open | ~130 MB peak (worker process only) |
| Medium | #9 Steam game HashMap | Open | ~7-10 MB steady, ~30 MB peak during refresh |
| Medium | #14 Rate limiter unbounded growth | Open | Slow leak over weeks/months |
| Medium | #13 DB tables without pruning | Open | Disk/query performance over time |
| Low | #10-12,#16-22 String allocation micro-patterns | Open | Bytes to low KB per occurrence |
| Low | #15 Oversized API cache capacities | Open | Wasted capacity reservation |

---

## Recommended Priority Actions

1. ~~**Fix the `Box::leak`** in `load_localization`~~ — **DONE**
2. ~~**Paginate `UserColor::find().all()`** in bot payload builders; use `.count()` in `info.rs`~~ — **DONE**
3. **Hash cache keys** instead of storing full GraphQL queries; reduce default capacity to 1,000-2,000
4. **Remove the `Arc<RwLock<>>` wrapper** around caches — Moka handles concurrency internally
5. **Pass the shared `reqwest::Client`** from `BotData` to shared library request functions
6. **Remove or lazy-load guild member chunking** -- only chunk when needed
7. **Configure serenity `CacheSettings`** with `max_messages` limit
8. **Add DB cleanup tasks** for `command_usage` and `ping_history`
