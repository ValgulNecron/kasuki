# Memory Usage Report

Analysis of the Kasuki bot codebase for memory-heavy patterns, potential leaks, and optimization opportunities.

**Last updated:** 2026-03-15

---

## High-Severity Issues

### 2. `guild_cache.members.clone()` Clones Entire Members HashMap

**File:** `bot/src/server_image/calculate_user_color.rs:15-16`

```rust
let members = guild_cache.members.clone();
return members.into_iter().map(|m| m.into()).collect();
```

When generating server images, the entire `HashMap<UserId, Member>` from the guild cache is cloned. For a guild with 1,000 members, this duplicates all user data, roles, nicknames, etc.

**Fix:** Iterate over `guild_cache.members` directly without cloning the entire HashMap. Use `.iter().map(|(_, m)| ...)` and collect only the fields needed (user_id, face URL).

### 3. `user_blacklist.read().await.clone()` Clones Full HashSet Into Task Payloads

**Files:** `bot/src/server_image/generate_server_image.rs:33,73`

```rust
let user_blacklist = bot_data.user_blacklist.read().await.clone();
```

The entire `HashSet<String>` is cloned and serialized into every `ImageTask::GenerateServerImage` payload. This happens twice per guild (local + global image).

**Fix:** Pass the blacklist as `Arc<RwLock<HashSet<String>>>` to the worker, or filter members before building the task payload so the blacklist doesn't need to be serialized.

### 4. `interaction.clone()` Clones Entire Interaction Enum

**File:** `bot/src/handlers/interaction.rs:17`

```rust
match interaction.clone() {
    Interaction::Command(command_interaction) => { ... }
    Interaction::Autocomplete(autocomplete_interaction) => { ... }
    Interaction::Component(component_interaction) => { ... }
    _ => {},
}
```

The full `Interaction` enum (which contains `CommandInteraction`, `ComponentInteraction`, etc. with all their fields) is cloned just to destructure it. This happens on **every interaction** the bot processes.

**Fix:** Use `match interaction { ... }` directly — the `interaction` is already owned (`Interaction`, not `&Interaction`), so the match can consume it. Extract `user` before the match or from each arm by reference.

### 5. `member.user.clone()` Called Twice in Same Handler

**File:** `bot/src/handlers/guild.rs:105,129`

```rust
enqueue_user_color(
    user_blacklist_server_image,
    member.user.clone(),  // clone 1
    bot_data.clone(),
).await;
// ... later ...
let user = member.user.clone();  // clone 2
if let Err(e) = add_user_data_to_db(user, bot_data.db_connection.clone()).await {
```

The `User` struct (with avatar, discriminator, global_name, etc.) is cloned twice in `guild_member_addition`. In `guild_members_chunk` (line 157-159), `user` is cloned then immediately cloned **again** for the DB call.

**Fix:** Clone once, reuse. Or refactor `enqueue_user_color` and `add_user_data_to_db` to take `&User` instead of owned `User`.

---

## Medium-Severity Issues

### 6. Database Tables With Unbounded Growth

| Table | File | Issue |
|---|---|---|
| `command_usage` | `bot_data.rs:73-88` | Inserts a row per command invocation, never pruned |
| `ping_history` | `launch_task/ping_manager.rs` | Inserts per shard per tick (~5,760 rows/day with 4 shards) |
| `vocal` | Voice session handler | Sessions inserted on leave, never cleaned |

No cleanup tasks have been added. The existing launch tasks (`game_management`, `ping_manager`, `user_blacklist`, `bot_info_update`, `queue_publisher`) contain no pruning logic.

**Fix:** Add a periodic cleanup task that deletes rows older than N days, or use partitioned tables with automatic drop of old partitions.

### 7. Rate Limiter `DashMapStateStore` Has No Eviction

**File:** `api-server/src/api/rate_limit.rs`

The `governor` rate limiter uses a `DashMapStateStore<String>` keyed by IP address. There is no eviction strategy -- the map grows unboundedly as unique IPs make requests over the server's lifetime.

**Fix:** Use `governor`'s `keyed::DefaultKeyedRateLimiter` with periodic cleanup, or switch to a time-bounded store.

### 8. `compare.rs` Clones Statistics Structs Repeatedly in `get_affinity`

**File:** `bot/src/command/anilist_user/compare.rs:470-501`

```rust
let anime = s1.anime.clone().unwrap();
let anime2 = s2.anime.clone().unwrap();
affinity = jaccard_index(
    &tag_string(&anime.tags.clone().unwrap()),    // clone tags
    &tag_string(&anime2.tags.clone().unwrap()),   // clone tags
);
affinity += jaccard_index(
    &genre_string(&anime.genres.clone().unwrap()), // clone genres
    &genre_string(&anime2.genres.clone().unwrap()), // clone genres
);
let manga = s1.manga.clone().unwrap();
// ... same pattern for manga ...
```

The anime/manga stats are cloned from `s1`/`s2`, then `tags` and `genres` are cloned again from the already-cloned structs. 8 unnecessary clones.

Additionally, `tag_string()` (line 867-875) and `genre_string()` (line 913-921) each clone every element:
```rust
let tag = tag.clone().unwrap();
tag.tag.unwrap().name.clone()
```

**Fix:** Take `s1`/`s2` by value (already consumed). Use `as_ref()` chains instead of clone in `tag_string`/`genre_string`:
```rust
fn tag_string(vec: &[Option<UserTagStatistic>]) -> Vec<String> {
    vec.iter()
        .filter_map(|tag| Some(tag.as_ref()?.tag.as_ref()?.name.clone()))
        .collect()
}
```

### 9. Redundant `.clone()` on Arc Before Deref in Loop

**File:** `bot/src/handlers/guild.rs:181`

```rust
.exec(&*db_connection.clone())
```

Inside `guild_members_chunk`, which loops over all members, `db_connection` (an `Arc<DatabaseConnection>`) is cloned then immediately dereferenced. The clone is wasted.

**Fix:** `.exec(&*db_connection)` — dereferences the Arc directly.

### 10. `bytes.clone()` and `name.clone()` in AI Image Attachment Loop

**File:** `bot/src/command/ai/image.rs:143-158`

```rust
for attachement in attachments {
    let name = attachement.1;
    let bytes = attachement.0;
    command_files.push(CommandFiles::new(name.clone(), bytes.clone()));  // clone name + entire image bytes
    embed_contents.push(
        embed_content.clone()
            .images_url(format!("attachment://{}", name.clone())),  // clone name again
    );
    // ... then saves bytes to storage
    if let Err(e) = image_store.save(&storage_key, &bytes).await {
```

Each image's bytes (potentially MB-sized) are cloned for `CommandFiles`, then the original is used for storage. The `name` is cloned twice per iteration.

**Fix:** Use `name` and `bytes` by moving them. Save to storage first, then move into `CommandFiles`. Or restructure to avoid double ownership.

---

## Low-Severity Issues

### 11. `if x != String::new()` Allocates for Comparison

**File:** `bot/src/command/anilist_user/staff.rs:138,149`

```rust
if date_of_birth != String::new() { ... }
if date_of_death != String::new() { ... }
```

**Fix:** Use `!date_of_birth.is_empty()`.

### 12. `anyhow!(format!(...))` Double Allocation

**Files:**
- `bot/src/command/admin/anilist/add_activity.rs:198`
- `bot/src/command/admin/anilist/delete_activity.rs:46,107`
- `bot/src/command/server/generate_image_pfp_server.rs:51`

**Fix:** `anyhow!("message {}", var)` accepts format args directly.

### 13. `if let Err(_)` Discards Error Information

**File:** `bot/src/server_image/calculate_user_color.rs:88`

```rust
if let Err(_) = bot_data.user_color_task_tx.send(task) {
    error!("User color queue publisher stopped, dropping task for user {}", user.id);
}
```

**Fix:** `if bot_data.user_color_task_tx.send(task).is_err()` or log the error: `if let Err(e) = ... { error!("...: {}", e); }`

### 14. `String::from("local")`/`String::from("global")` for Image Type

**File:** `bot/src/server_image/generate_server_image.rs:47,79`

```rust
image_type: String::from("local"),
image_type: String::from("global"),
```

**Fix:** Consider using an enum for image type instead of runtime strings, or at minimum `"local".to_owned()`.

---

## Architecture Notes

### Image Generation Worker

Server image generation and user color calculation run in a dedicated `image_generation` worker binary. Communication happens via Redis queue (mpsc channels on the bot side).

**What runs in the worker:**
- Color calculation (CIELAB Delta-E 2000 matching)
- Mosaic generation (4096x4096 canvas compositing via rayon)
- Image saving to S3-compatible storage (two versions: 128x128 thumbnail + 4096x4096 full-size)
- DB upserts for `user_color` and `server_image` tables

**What stays in the bot:**
- Guild member fetching from Discord API/cache
- Publishing lightweight `ImageTask` to Redis (IDs + avatar URLs only, no color data)

The worker's ~130 MB peak for the 4096x4096 canvas is inherent to the mosaic algorithm and acceptable.

---

## Summary by Estimated Memory Impact

| Priority | Issue | Status | Estimated Impact (bot process) |
|----------|-------|--------|-------------------------------|
| High | #1 Guild member chunking (all members, all guilds) | Open | Hundreds of MB for large bots |
| High | #2 `guild_cache.members.clone()` in server image gen | Open | Proportional to guild size |
| High | #3 Blacklist HashSet cloned into every image task | Open | Scales with blacklist size × guilds |
| High | #4 `interaction.clone()` on every interaction | Open | Hot path, many KB per interaction |
| High | #5 `member.user.clone()` called twice in guild handler | Open | Hot path in member events |
| Medium | #7 Rate limiter unbounded growth | Open | Slow leak over weeks/months |
| Medium | #6 DB tables without pruning | Open | Disk/query performance over time |
| Medium | #8 compare.rs clones stats structs 8+ times | Open | KB per compare command |
| Medium | #9 Redundant Arc clone in loop | Open | Trivial per-call, adds up in chunk |
| Medium | #10 Image bytes cloned in attachment loop | Open | MB per AI image command |
| Low | #11-14 String allocation micro-patterns | Open | Bytes per occurrence |

---

## Recommended Priority Actions

1. **Remove `interaction.clone()`** — consume the owned value directly in the match
2. **Avoid full `members.clone()`** — iterate by reference, collect only needed fields
3. **Remove or lazy-load guild member chunking** — only chunk when needed
4. **Refactor `enqueue_user_color` / `add_user_data_to_db`** to take `&User` — eliminates double clones
5. **Configure serenity `CacheSettings`** with `max_messages` limit
6. **Add DB cleanup tasks** for `command_usage` and `ping_history`
7. **Fix compare.rs `get_affinity`** — use references instead of cloning stats structs
