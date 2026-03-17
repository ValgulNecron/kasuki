# Old Convention Report

Audit of the Kasuki bot codebase for anti-patterns, old conventions, and areas needing cleanup.

**Last updated:** 2026-03-16

---

## Completed Fixes (issues #1–#23)

All 23 original issues have been resolved. Summary:

| #  | Issue                                    | Fix                                                               |
|----|------------------------------------------|-------------------------------------------------------------------|
| 1  | `DEFAULT_STRING` was `&String`           | Changed to `&str = ""`                                            |
| 2  | Hardcoded AI rate limits                 | Moved to `config.toml` `[ai.rate_limits]`                         |
| 3  | `reqwest::Client::new()` per request     | Shared via `Arc<Client>` in `BotData`                             |
| 4  | Duplicated DB connection logic           | All crates use `config.db.connect().await`                        |
| 5  | `process::exit()` instead of `?`         | `run() -> Result`, exit only in `main()` wrapper                  |
| 6  | `unsafe { set_var() }`                   | Passed via `Command::env()` to child process                      |
| 7  | Unnecessary `.clone()` on Arc contents   | References (`&cx.config`, `&cx.http_client`)                      |
| 8  | `bot_data.clone()`                       | N/A — Arc clone is cheap ref-count increment                      |
| 9  | Silent error swallowing                  | `remove_test_sub` counts failures; OAuth redirects on error       |
| 10 | Unhashed cache keys                      | N/A — `CacheInterface` hashes internally                          |
| 11 | `Arc<RwLock<>>` around caches            | Removed; caches are `Arc<CacheInterface>`                         |
| 12 | `ctx.clone()` in handlers                | All handlers + autocomplete chain take `&SerenityContext`         |
| 13 | Duplicate server image commands          | Consolidated to single implementation                             |
| 14 | `#[allow(dead_code)]` in API structs     | Removed unused fields from `UserInfo`, `Guild`, `RawDiscordGuild` |
| 15 | quality/style swap bug                   | Conditional insertion in `build_image_payload()`                  |
| 16 | Verbose match in AI image                | Extracted to `shared::service::ai::build_image_payload()`         |
| 17 | Hardcoded Discord API URLs               | `DISCORD_API_BASE` and `DISCORD_OAUTH_AUTHORIZE` constants        |
| 18 | Missing config validation                | `validate()` on `Config` and `WorkerConfig` after parsing         |
| 19 | `!= String::new()` comparisons           | Replaced with `.is_empty()`                                       |
| 20 | `push_str(format!().as_str())`           | Replaced with `write!()` from `std::fmt::Write`                   |
| 21 | Excessive `.clone()` in AniList builders | References, moves, `as_ref()`                                     |
| 22 | Rate limiter memory leak                 | `spawn_rate_limiter_cleanup()` every 5 min                        |
| 23 | Oversized cache capacities               | Reduced `auth_codes`/`oauth_states` to 1,000                      |
| 24 | `String::from` in map lookups            | Replaced all 35 `.get(&String::from("x"))` with `.get("x")`       |
| 25 | Full `Interaction` clone                 | Consume owned `interaction` directly; move `.user` instead of clone |
| 26 | `clone().unwrap_or_default()` chains     | `as_deref()` in image.rs; move in staff.rs; `filter_map` in compare.rs |
| 27 | `#[allow(dead_code)]` in bot crate       | Removed 17 dead items; kept `#[allow]` on 12 cynic query-chain types  |

---

## Priority Summary

All issues resolved.

| Priority   | Issue                                    | Status   | Impact                    |
|------------|------------------------------------------|----------|---------------------------|
| **Medium** | #24 `String::from` in map lookups        | **Done** | 35 heap allocs per interaction |
| **Medium** | #25 Full `Interaction` clone             | **Done** | Clones all command data needlessly |
| **Low**    | #26 `clone().unwrap_or_default()` chains | **Done** | Unnecessary String allocs |
| **Low**    | #27 `#[allow(dead_code)]` in bot crate   | **Done** | Code cleanliness          |
