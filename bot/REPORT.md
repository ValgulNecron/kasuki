# Memory Usage Report

Analysis of the Kasuki bot codebase for memory-heavy patterns, potential leaks, and optimization opportunities.

**Last updated:** 2026-03-08

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

### ~~3. AniList/VNDB Caches: Partially Improved, Remaining Issues~~ (FIXED)

**Status:** Resolved.

**What's been fixed (2025-11):**
- `CacheConfig` now has configurable fields: `cache_type` (memory/redis), `max_capacity`, `ttl_secs`, `host`, `port`, `password`
- Redis backend added as alternative to Moka (in-memory), selectable via `config.toml`
- `CacheInterface::from_config()` respects configuration settings
- Graceful fallback: if Redis connection fails, falls back to in-memory cache automatically

**What's been fixed (2026-03):**
- Cache keys are now **hashed to fixed 32-char hex strings** (128-bit, two independent SipHash passes) inside `CacheInterface::read()`/`write()` — all callers benefit automatically. Collision probability for 10K entries: ~2.9×10⁻³¹
- Default capacity reduced from 10,000 to **2,000** entries (both `CacheInterface::new()` default and `CacheConfig` default)
- Removed `Arc<RwLock<>>` wrapper — caches are now `Arc<CacheInterface>` across all 3 crates (bot, worker, shared). Moka and Redis multiplexed connections handle concurrency internally. ~30 call sites simplified.

---

## High-Severity Issues

### ~~4. `reqwest::Client::new()` Created Per Request~~ (FIXED)

**Status:** Resolved. All HTTP client usage now reuses a shared pooled client:
- `shared/src/anilist/make_request.rs` — uses `static LazyLock<Client>` (shared across all AniList calls from shared lib and worker)
- `bot/src/helper/make_graphql_cached.rs` — already used `static LazyLock<Client>` for bot-side AniList calls
- `shared/src/vndb/common.rs` — already accepts `&reqwest::Client` parameter from callers
- `bot/src/command/ai/image.rs` — already uses `bot_data.http_client` for all requests

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

### ~~6. `user_blacklist.read().await.clone()` Copies Entire Vec~~ (FIXED)

**Status:** Resolved. Changed `user_blacklist` from `Vec<String>` to `HashSet<String>` across:
- `BotData.user_blacklist` field type
- `update_user_blacklist` task parameter and collection type
- `enqueue_user_color` parameter type
- `ImageTask::GenerateServerImage.blacklist` field type
- All callers already use `.contains()` which is now O(1) instead of O(n)

---

### ~~7. `lava_client.read().await.clone()` in Every Music Command~~ (FIXED)

**Status:** Resolved. Changed `lavalink` field from `Arc<RwLock<Option<LavalinkClient>>>` to `Arc<RwLock<Option<Arc<LavalinkClient>>>>`. Cloning the inner value is now a single `Arc` refcount bump instead of allocating a new `Vec<Arc<Node>>` + bumping multiple `Arc` refcounts per music command.

---

## Medium-Severity Issues

### 8. Server Image Generation: 64 MB Canvas + Bulk Member Clone (WON'T FIX)

**File:** `image_generation/src/mosaic.rs` (worker)

- A `DynamicImage::new_rgba8(4096, 4096)` allocates **64 MB** of RGBA pixel data — inherent to the 4096×4096 output resolution
- All avatar images must be held simultaneously because `find_closest_color_index` picks from the full set for each pixel — the same tile can be reused for multiple pixels, so streaming/batching isn't possible without fundamentally changing the algorithm

**Why won't fix:** Both the canvas and the avatar set are required by the mosaic algorithm. This runs in a dedicated worker process, not the bot. The ~130 MB peak is acceptable for the worker's purpose.

### ~~9. Steam Game HashMap (~7-10 MB, 3x Peak During Refresh)~~ (FIXED)

**Status:** Resolved.
- `reqwest::get()` (new client per call) replaced with a `static LazyLock<Client>` shared HTTP client
- Response is now deserialized directly from the response stream via `.json::<AppListResponse>()` — no intermediate `String` or `serde_json::Value`
- Removed `.clone()` of the JSON apps array — typed struct is consumed directly
- Changed `u128` to `u32` for Steam app IDs (saves 12 bytes per entry × ~170K entries = ~2 MB)
- `Vec<App>` consumed via `.into_iter()` to build the HashMap — no cloning of game names
- Peak memory during refresh reduced from ~3x to ~2x steady state

### ~~10. `push_str(format!(...).as_str())` -- Unnecessary Temp String~~ (FIXED)

**Status:** Resolved. All `push_str(format!(...).as_str())` replaced with `write!()` / `writeln!()` from `std::fmt::Write`:
- `character.rs` — 3 date formatting calls
- `media.rs` — staff text, character text, and 3 song link calls
- Also fixed `!= String::new()` comparisons to `!.is_empty()` in `media.rs` (issue #17)

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

### ~~15. API Server Cache Capacities Oversized~~ (PARTIALLY FIXED)

**Status:** `auth_codes` and `oauth_states` reduced from 10,000 to 1,000. `user_cache` remains at 10,000 (reasonable given 24h TTL).

**File:** `api-server/src/api/state.rs`

| Cache | Capacity | TTL | Status |
|---|---|---|---|
| `user_cache` | 10,000 | 24h | Acceptable |
| `auth_codes` | 1,000 | 5m | **FIXED** (was 10,000) |
| `oauth_states` | 1,000 | 10m | **FIXED** (was 10,000) |

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

### ~~20. Minigame Inventory: Cloning Vec of DB Structs to Sort~~ (FIXED)

**Status:** Resolved.
- `inventory.rs` — Two `fish_items.clone()` replaced with `Vec<usize>` index sorting (rarity/size sort and name sort)
- `fish_inventory.rs` — `fish_list.clone()` + sort + `.first()` replaced with `iter().max_by()` (zero allocation)
- `fish_inventory.rs` — Removed dead `all_fish = inventory_items.clone()` that was sorted but never used

### ~~21. Unused Clone~~ (FIXED)

**Status:** Resolved. Removed `let _config = bot_data.config.clone();` from `list_all_activity.rs`.

### ~~22. Redundant `.clone()` at End of Builder Chain~~ (FIXED)

**Status:** Resolved. Changed `add_files` signature from `&mut self → &mut Self` to `self → Self` (consuming builder pattern, matching `action_row`). Removed redundant `.clone()` at 2 call sites and converted 3 other callers from mutable-then-return to chained builder style.

---

## Architecture Change: Image Generation Worker

Server image generation and user color calculation run in a dedicated `image_generation` worker binary. Communication happens via Redis queue (mpsc channels on the bot side).

**What runs in the worker:**
- Color calculation (CIELAB Delta-E 2000 matching)
- Mosaic generation (4096x4096 canvas compositing via rayon)
- Image saving to S3-compatible storage (two versions: 128x128 thumbnail for mosaic tiles + 4096x4096 full-size for display)
- DB upserts for `user_color` and `server_image` tables
- DB queries for `UserColor` records (worker fetches its own data)

**What stays in the bot:**
- Guild member fetching from Discord API/cache
- Publishing lightweight `ImageTask` to Redis (IDs + avatar URLs only, no color data)
- For local server images: sends `(user_id, profile_picture_url)` per member
- For global server images: sends empty `members` vec — worker fetches all `UserColor` from DB

**Impact on memory issues:**
- Issues #2, #6, and #8 are **resolved/mitigated** for the bot process
- The bot no longer loads the `user_color` table at all — worker handles all DB queries
- Worker does a single bulk query for global images, or batch-filtered query for local images
- `user_blacklist` uses `HashSet<String>` for O(1) lookups and is serialized into the task payload

---

## Summary by Estimated Memory Impact

| Priority     | Issue                                                                 | Status        | Estimated Impact (bot process)                  |
|--------------|-----------------------------------------------------------------------|---------------|-------------------------------------------------|
| ~~Critical~~ | ~~#1 `Box::leak` in `load_localization`~~                             | **FIXED**     | ~~Unbounded growth (leak)~~                     |
| ~~Critical~~ | ~~#2 `UserColor::find().all()` in bot (payload + info command)~~      | **FIXED**     | ~~Up to several GB~~                            |
| ~~Critical~~ | ~~#3 AniList/VNDB caches (keys unhashed, double-wrapped, oversized)~~ | **FIXED**     | ~~150-300 MB~~                                  |
| High         | #5 Guild member chunking (all members, all guilds)                    | Open          | Hundreds of MB for large bots                   |
| ~~High~~     | ~~#6 `user_blacklist` Vec clone~~                                     | **FIXED**     | ~~O(n) contains + full Vec clone~~              |
| ~~High~~     | ~~#4 `reqwest::Client::new()` per request (shared lib)~~              | **FIXED**     | ~~Connection pool churn (indirect)~~            |
| Medium       | #8 Server image 64 MB canvas + avatars                                | **WON'T FIX** | ~130 MB peak (worker process, by design)        |
| ~~Medium~~   | ~~#9 Steam game HashMap~~                                             | **FIXED**     | ~~~7-10 MB steady, ~30 MB peak during refresh~~ |
| Medium       | #14 Rate limiter unbounded growth                                     | Open          | Slow leak over weeks/months                     |
| Medium       | #13 DB tables without pruning                                         | Open          | Disk/query performance over time                |
| Low          | #10 push_str(format!())                                               | **FIXED**     | Bytes per occurrence                            |
| Low          | #11-12,#16-22 String allocation micro-patterns                        | Open          | Bytes to low KB per occurrence                  |
| ~~Low~~      | ~~#15 Oversized API cache capacities~~                                | **PARTIALLY FIXED** | ~~auth_codes/oauth_states reduced to 1K~~  |

---

## Recommended Priority Actions

1. ~~**Fix the `Box::leak`** in `load_localization`~~ — **DONE**
2. ~~**Paginate `UserColor::find().all()`** in bot payload builders; use `.count()` in `info.rs`~~ — **DONE**
3. ~~**Hash cache keys** instead of storing full GraphQL queries; reduce default capacity to 1,000-2,000~~ — **DONE**
4. ~~**Remove the `Arc<RwLock<>>` wrapper** around caches — Moka handles concurrency internally~~ — **DONE**
5. ~~**Pass the shared `reqwest::Client`** from `BotData` to shared library request functions~~ — **DONE**
6. **Remove or lazy-load guild member chunking** -- only chunk when needed
7. **Configure serenity `CacheSettings`** with `max_messages` limit
8. **Add DB cleanup tasks** for `command_usage` and `ping_history`
