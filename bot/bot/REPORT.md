# Memory Usage Report

Analysis of the Kasuki bot codebase for memory-heavy patterns, potential leaks, and optimization opportunities.

---

## Critical Issues

### 1. Memory Leak via `Box::leak` in `load_localization`

**File:** `shared/src/localization.rs:296`

```rust
let json: &'a str = Box::leak(json_content.into_boxed_str());
```

Every call to `load_localization()` reads a JSON file from disk, allocates a `String`, and then **permanently leaks it** via `Box::leak()`. The allocation is never reclaimed. This function is called from:
- `worker/src/activity/anime_activity.rs:96` (called in a loop per notification)
- Multiple command handlers

Each invocation leaks a few KB. Over hours/days of uptime this grows unboundedly.

**Fix:** Use the Fluent `USABLE_LOCALES` static loader (already used by newer code) for all localization. If JSON loading is still needed, parse with an owned `String` instead of leaking into `'static`.

---

### 2. `UserColor::find().all()` Loads Entire Table into RAM

**File:** `bot/src/server_image/generate_server_image.rs:49` (bot), `image_generation/src/main.rs:109` (worker)

The global server image path calls `UserColor::find().all(&*connection).await?`, loading **every row** from the `user_color` table into memory at once. Each row contains a base64-encoded 128x128 PNG image (~25-30 KB of text per user). For a bot serving 100,000 users, this is **2.5-3 GB** loaded into a single `Vec`.

**Update (2025-02):** Image generation has been moved to a dedicated `image_generation` worker binary communicating via Redis queue. The bot now pre-fetches the color map and sends it as part of the task payload, so this load no longer blocks the bot's async runtime. However, the memory spike still occurs in two places: the bot (for building the task payload in `enqueue_global_server_image`) and the worker (during mosaic generation). The worker is a separate process, so this no longer degrades bot responsiveness.

**Fix:** Use `.paginate(page_size)` or `.find().stream()` to process rows in batches. In the bot, consider sending only user IDs in the task and letting the worker fetch color data itself in batches.

---

### 3. AniList/VNDB Caches: Oversized and Non-Configurable

**File:** `shared/src/cache.rs:12-15`

```rust
Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(3600))
    .build()
```

Both caches store `String -> String` (full GraphQL query as key, full JSON response as value). AniList responses can be 2-10 KB each. At max capacity:
- **10,000 entries x ~5 KB average = ~50-100 MB per cache**
- **3 instances** exist (bot anilist, bot vndb, worker anilist) = up to **150-300 MB total**

The cache keys are the **full GraphQL query string** concatenated with serialized variables (hundreds of bytes per key), wasting memory on key storage alone.

`CacheConfig` in `config.rs` is an **empty struct** -- capacity and TTL are hardcoded with no way to tune them from `config.toml`.

**Fix:**
- Hash the query+variables into a fixed-size key (e.g., `u64` via `xxhash` or `seahash`)
- Make `max_capacity` and `time_to_live` configurable via `config.toml`
- Reduce default capacity to 1,000-2,000 (more realistic for a Discord bot)
- Remove the `Arc<RwLock<>>` wrapper around Moka caches -- Moka is already `Clone + Send + Sync` internally; the RwLock adds unnecessary contention

---

## High-Severity Issues

### 4. `reqwest::Client::new()` Created Per Request

Creating a new HTTP client per request discards the connection pool and forces a fresh TLS handshake every time.

| File | Line | Context |
|---|---|---|
| `shared/src/anilist/make_request.rs` | 104 | Every AniList API call via the shared library |
| `shared/src/vndb/common.rs` | 20, 61 | Every VNDB API call |
| `bot/src/command/ai/image.rs` | 290 | Inside a loop downloading AI-generated images |

**Fix:** Store a shared `reqwest::Client` in `BotData` (or use a `LazyLock<Client>` static as `make_graphql_cached.rs` already does) and pass it through to all request functions.

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

**File:** `bot/src/server_image/generate_server_image.rs:47`

```rust
let user_blacklist = bot_data.user_blacklist.read().await.clone();
```

Clones the entire `Vec<String>` blacklist out of the RwLock. The clone is serialized into the Redis task payload and sent to the `image_generation` worker. This is unavoidable for the queue-based architecture since the data must be serialized, but the Vec itself is still cloned from the RwLock.

**Fix:** Switch to an `Arc<HashSet<String>>` that can be cheaply cloned via Arc. The serialization into the task payload is necessary, but holding the read guard while serializing (instead of cloning first) would avoid the extra allocation.

### 7. `lava_client.read().await.clone()` in Every Music Command

**Files:** All 10 music command files (`clear.rs`, `leave.rs`, `pause.rs`, `queue.rs`, `remove.rs`, `resume.rs`, `seek.rs`, `skip.rs`, `stop.rs`, `swap.rs`)

```rust
let lava_client = lava_client.read().await.clone();
```

Clones the inner `Option<LavalinkClient>` on every music command invocation. If `LavalinkClient` is not just an Arc wrapper, this duplicates internal state.

**Fix:** Hold the read guard for the duration of the operation, or ensure the inner type is `Arc`-wrapped so cloning is a pointer bump.

---

## Medium-Severity Issues

### 8. Server Image Generation: 64 MB Canvas + Bulk Member Clone

**File:** `image_generation/src/mosaic.rs` (worker), `bot/src/server_image/calculate_user_color.rs:18` (bot)

- A `DynamicImage::new_rgba8(4096, 4096)` allocates **64 MB** of RGBA pixel data
- `guild_cache.members.clone()` clones all `Member` structs from the serenity cache into a `Vec`
- Per-user avatar images (128x128, ~65 KB each) are decoded and held simultaneously

For a 1,000-member guild, peak memory during generation is **~130 MB** (64 MB canvas + 65 MB decoded avatars).

**Update (2025-02):** The 64 MB canvas allocation and image compositing now happen in the dedicated `image_generation` worker process, not the bot. The bot still clones guild members from the serenity cache (via `get_member()`) to build the task payload, but the heavy image processing is fully isolated. This means the bot's memory footprint during server image operations is now limited to the member list + task serialization, while the worker handles the 64 MB canvas independently.

**Fix:**
- Process members in batches rather than cloning all at once
- Consider a smaller canvas (2048x2048 = 16 MB) unless the resolution is essential
- Stream and composite avatar tiles incrementally

### 9. Steam Game HashMap (~7-10 MB, 3x Peak During Refresh)

**File:** `bot/src/structure/steam_game_id_struct.rs:41-138`

The Steam API returns ~170,000 apps. During refresh, the raw JSON body, the parsed `Vec<App>`, and the new `HashMap` all exist simultaneously before the old map is dropped. Peak memory is ~3x the steady-state ~7-10 MB.

**Fix:**
- Parse directly from the response stream into the HashMap (avoid intermediate `Vec<App>`)
- Consider using `u32` instead of `u128` for app IDs (Steam app IDs fit in u32)
- Use `CompactString` or intern game names if memory is critical

### 10. `push_str(format!(...).as_str())` -- Unnecessary Temp String

**Files:**
- `bot/src/structure/run/anilist/character.rs:105, 115, 125`
- `bot/src/structure/run/anilist/media.rs:394, 433, 462, 465, 467, 571, 575, 595, 599`
- `bot/src/structure/run/anilist/user.rs:453, 461, 469, 477`

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

Same pattern in `compare.rs:866-918`.

### 12. `vn.image.clone()` Called Twice Then Moved

**Files:** `bot/src/command/vn/game.rs:139-150`, `bot/src/command/vn/character.rs:193-208`

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
| `ping_history` | `launch_task/ping_manager.rs:111-122` | Inserts per shard per tick (~5,760 rows/day with 4 shards) |
| `vocal` | Voice session handler | Sessions inserted on leave, never cleaned |

**Fix:** Add a periodic cleanup task that deletes rows older than N days, or use partitioned tables with automatic drop of old partitions.

### 14. Rate Limiter `DashMapStateStore` Has No Eviction

**File:** `api-server/src/api/rate_limit.rs:18`

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

**Files:** `bot/src/command/anilist_user/staff.rs:136,147`, `bot/src/structure/run/anilist/media.rs:461,464,466`

**Fix:** Use `!x.is_empty()`.

### 18. `anyhow!(format!(...))` Double Allocation

**Files:** `bot/src/command/admin/anilist/add_activity.rs:209`, `bot/src/command/admin/anilist/delete_activity.rs:59,120`, `bot/src/command/ai/image.rs:274`

**Fix:** `anyhow!("message {}", var)` accepts format args directly.

### 19. `Vec::from(body.clone())` on Bytes

**File:** `bot/src/command/ai/image.rs:301`

`Bytes::clone()` is cheap (Arc bump), but `Vec::from()` copies the entire buffer. The `.clone()` before `Vec::from()` is redundant.

### 20. Minigame Inventory: Cloning Vec of DB Structs to Sort

**Files:** `bot/src/command/minigame/fish_inventory.rs:122`, `bot/src/command/minigame/inventory.rs:118,196`

```rust
let mut sorted_fish = fish_list.clone();
sorted_fish.sort_by(...)
```

Clones a `Vec<(UserInventoryModel, ItemModel)>` (full DB row structs) just to sort.

**Fix:** Sort a `Vec<usize>` of indices, or take ownership of the data instead of borrowing and cloning.

### 21. Unused Clone

**File:** `bot/src/command/admin/anilist/list_all_activity.rs:64`

```rust
let _config = bot_data.config.clone(); // never used
```

### 22. Redundant `.clone()` at End of Builder Chain

**Files:** `bot/src/command/ai/image.rs:160-162`, `bot/src/command/anilist_user/seiyuu.rs:222-224`

```rust
EmbedsContents::new(...).add_files(command_files).clone(); // clone of owned value
```

---

## Architecture Change (2025-02): Image Generation Worker

Server image generation and user color calculation have been moved from the bot process into a dedicated `image_generation` worker binary. Communication happens via a Redis queue (`image_generation:tasks` key, rpush/blpop pattern).

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
- Issue #2 still applies to the worker and to the bot's `enqueue_global_server_image` (loads full color table for payload)
- The bot gains a small Redis connection overhead (~few KB)

---

## Summary by Estimated Memory Impact

| Priority | Issue | Estimated Impact (bot process) |
|---|---|---|
| Critical | `Box::leak` in `load_localization` | Unbounded growth (leak) |
| Critical | `UserColor::find().all()` in global image gen | Up to several GB (now mitigated: loads in bot for payload, heavy processing in worker) |
| Critical | AniList/VNDB caches (3x 10,000 entries) | 150-300 MB |
| High | Guild member chunking (all members, all guilds) | Hundreds of MB for large bots |
| High | `reqwest::Client::new()` per request | Connection pool churn (indirect) |
| Medium | Server image 64 MB canvas + member clones | ~130 MB peak (now in worker process, not bot) |
| Medium | Steam game HashMap | ~7-10 MB steady, ~30 MB peak during refresh |
| Medium | Rate limiter unbounded growth | Slow leak over weeks/months |
| Medium | DB tables without pruning | Disk/query performance over time |
| Low | String allocation micro-patterns | Bytes to low KB per occurrence |
| Low | Oversized API cache capacities | Wasted capacity reservation |

---

## Recommended Priority Actions

1. **Fix the `Box::leak`** in `load_localization` -- this is a real memory leak
2. **Paginate `UserColor::find().all()`** in both bot payload builder and worker -- still loads full table
3. **Hash cache keys** instead of storing full GraphQL queries
4. **Make cache capacity configurable** and reduce defaults to 1,000-2,000
5. **Share a single `reqwest::Client`** across all HTTP-calling code
6. **Remove or lazy-load guild member chunking** -- only chunk when needed
7. **Configure serenity `CacheSettings`** with `max_messages` limit
8. **Add DB cleanup tasks** for `command_usage` and `ping_history`
