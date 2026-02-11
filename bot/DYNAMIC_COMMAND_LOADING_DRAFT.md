# Draft: Better Dynamic Command Loading for Kasuki

## Current Pain Points

1. **Giant match statement** in `command_dispatch.rs` (~900 lines) — every new command requires adding an import, a
   match arm, and a JSON file
2. **JSON files are disconnected from code** — the 155 JSON files in `json/` define command metadata separately from the
   Rust handlers, so they can drift out of sync
3. **No compile-time guarantees** — a typo in a JSON file or a missing match arm only shows up at runtime
4. **Boilerplate per command** — each command needs: a struct, `impl_command!` macro call, a JSON file, and a dispatch
   match arm
5. **Massive locale duplication** — every JSON file repeats locale strings that already exist (or should exist) in
   Fluent `.ftl` files

## Discord's 3-Level Command Hierarchy

Discord supports three levels of nesting for slash commands:

```
Level 1: /command                        (e.g. /anime, /register)
Level 2: /command subcommand             (e.g. /bot credit, /ai image)
Level 3: /command group subcommand       (e.g. /admin anilist add_anime_activity)
```

The current codebase maps these to:

- **Level 1** — `json/command/*.json` (top-level commands like `anime.json`)
- **Level 2** — `json/subcommand/*.json` (groups like `bot.json` containing `credit`, `info`, `ping`)
- **Level 3** — `json/subcommand_group/*.json` (nested groups like `admin.json` → `anilist` → `add_anime_activity`)

Any replacement must cleanly support all three levels.

## Proposed Approach: Convention-Over-Configuration Proc Macro

The key insight: **most metadata can be inferred** from function names, parameter types, module structure, and existing
Fluent translation files. Only annotate what deviates from defaults.

### Design Principles

1. **Name = function name** — no need to repeat it
2. **Description = Fluent lookup** — you already have `.ftl` files; reuse them for registration too
3. **Arg type = Rust type** — `String`, `i64`, `bool`, `Option<T>` map directly to Discord types
4. **Required = not Option** — `String` is required, `Option<String>` is optional
5. **Defaults cover 90% of commands** — `nsfw=false`, all contexts enabled, no special permissions

### Level 1: Top-Level Commands (`/anime`)

```rust
// bot/src/command/anilist_user/anime.rs

#[command]
pub async fn anime(ctx: CommandContext, #[autocomplete] anime_name: String) -> Result<EmbedsContents> {
    ctx.defer().await?;
    // ... existing logic ...
}
```

That's it. The macro infers:

- `name` = `"anime"` (from function name)
- `description` = looked up from `translation/en-US/anilist_user_anime.ftl` key `command-desc`
- `anime_name` arg: type `String`, required (not `Option`), autocomplete enabled
- All locale names/descriptions: scanned from `translation/*/anilist_user_anime.ftl`
- `nsfw` = `false`, contexts = all, install_contexts = all (defaults)

For the rare command that needs overrides:

```rust
#[command(nsfw = true, contexts = [Guild])]
pub async fn random_nsfw_image(ctx: CommandContext) -> Result<EmbedsContents> { ... }
```

### Level 2: Subcommands (`/bot credit`)

Use a module to group subcommands under a parent command. The module name becomes the command name.

```rust
// bot/src/command/bot/mod.rs

#[command_group]
mod bot {
    #[subcommand]
    pub async fn credit(ctx: CommandContext) -> Result<EmbedsContents> { ... }

    #[subcommand]
    pub async fn info(ctx: CommandContext) -> Result<EmbedsContents> { ... }

    #[subcommand]
    pub async fn ping(ctx: CommandContext) -> Result<EmbedsContents> { ... }
}
```

The `#[command_group]` macro:

- Registers `/bot` as a top-level command with 3 subcommands
- Group name = module name `"bot"`
- Group description = from `translation/en-US/bot.ftl` key `group-desc`
- Each `#[subcommand]` name = function name, description = from Fluent
- Locales for group + each subcommand = auto-scanned from `.ftl` files

This replaces `json/subcommand/bot.json` (116 lines) with ~10 lines of Rust.

### Level 3: Subcommand Groups (`/admin anilist add_anime_activity`)

Nest another module inside the command group to create the third level.

```rust
// bot/src/command/admin/mod.rs

#[command_group(permissions = [Administrator], contexts = [Guild])]
mod admin {
    #[subcommand_group]
    mod anilist {
        #[subcommand]
        pub async fn add_anime_activity(
            ctx: CommandContext,
            #[autocomplete] anime_name: String,
            delays: Option<i64>,
        ) -> Result<EmbedsContents> { ... }

        #[subcommand]
        pub async fn delete_anime_activity(
            ctx: CommandContext,
            #[autocomplete] anime_name: String,
        ) -> Result<EmbedsContents> { ... }
    }

    #[subcommand_group]
    mod general {
        #[subcommand]
        pub async fn lang(
            ctx: CommandContext,
            #[choices(en, jp, de, fr, "es-ES", "zh-CN", ru)]
            lang_choice: String,
        ) -> Result<EmbedsContents> { ... }

        #[subcommand]
        pub async fn module(
            ctx: CommandContext,
            #[choices(AI, ANILIST, GAME, ANIME, VN, LEVEL, MINIGAME)]
            name: String,
            state: bool,
        ) -> Result<EmbedsContents> { ... }
    }
}
```

This maps directly to Discord's hierarchy:

- `admin` → top-level command (Level 1)
- `anilist`, `general` → subcommand groups (Level 2)
- `add_anime_activity`, `lang`, etc. → subcommands (Level 3)

This replaces `json/subcommand_group/admin.json` (705 lines!) with ~30 lines of Rust.

### Arg Attributes for Special Cases

Most args need no annotation at all. For special behaviors, use small targeted attributes:

```rust
// Autocomplete
#[autocomplete] anime_name: String

// Choices (static list)
#[choices(en, jp, de, fr)] lang: String

// Optional arg (inferred from Option<T>)
delays: Option<i64>

// Override description (if Fluent lookup isn't desired)
#[desc = "A custom description"] filter: String

// Rename the arg as shown in Discord (if function param name differs)
#[name = "anime_name"] anime: String
```

### Localization: Reuse Fluent Files

The biggest boilerplate killer. Instead of duplicating locale data in attributes or JSON, the macro generates
registration code that reads from your existing Fluent `.ftl` files at startup.

**Fluent file convention** (add new keys to existing files):

```ftl
# translation/en-US/bot.ftl (existing file, add registration keys)

# Registration metadata (new keys, used by the macro at startup)
group-desc = Command to get information about the bot.

# Subcommand descriptions
credit-desc = Get the credit of the app.
info-desc = Get information on the bot.
ping-desc = Get the ping of the bot (and the shard id).
```

```ftl
# translation/fr/bot.ftl
group-name = bot
group-desc = Commande pour obtenir des informations sur le bot.

credit-name = credit
credit-desc = Obtenir le crédit de l'application.
# ...
```

The generated registration code does:

```rust
// Auto-generated by #[command_group]
fn build_bot_command(locales: &LocaleData) -> CreateCommand {
    let mut cmd = CreateCommand::new("bot")
        .description(locales.get("en-US", "bot", "group-desc"))
        .add_option(/* credit subcommand */)
        .add_option(/* info subcommand */)
        .add_option(/* ping subcommand */);

    // Auto-apply all available locales
    for locale in locales.available_locales("bot") {
        cmd = cmd
            .name_localized(locale, locales.get(locale, "bot", "group-name"))
            .description_localized(locale, locales.get(locale, "bot", "group-desc"));
    }
    cmd
}
```

No locale strings in Rust code — they live in `.ftl` files only. Add a new language by adding a new
`translation/{locale}/` folder, no Rust changes needed.

### Choice Localization

For `#[choices(...)]`, the choice display names are also pulled from Fluent:

```ftl
# translation/en-US/admin_general.ftl
lang-choice-en = English
lang-choice-jp = Japanese
lang-choice-de = German

module-choice-AI = AI
module-choice-ANILIST = ANILIST
```

```ftl
# translation/fr/admin_general.ftl
lang-choice-en = Anglais
lang-choice-jp = Japonais
lang-choice-de = Allemand

module-choice-AI = IA
module-choice-ANILIST = ANILIST
```

## Auto-Registration via Inventory

```rust
// shared or bot crate
pub struct CommandDescriptor {
    /// Dispatch key (e.g. "anime", "bot_credit", "admin_anilist_add_anime_activity")
    pub dispatch_name: &'static str,
    /// Which level this handler sits at
    pub kind: CommandKind, // Command, Subcommand, SubcommandGroup
    pub guild_only: bool,
    /// Builds the Discord CreateCommand (only called for top-level entries)
    pub build_fn: fn(&LocaleData) -> CreateCommand,
    /// Executes the command handler
    pub handler_fn: fn(SerenityContext, CommandInteraction) -> BoxFuture<'static, Result<()>>,
}

inventory::collect!(CommandDescriptor);
```

At startup:

```rust
pub async fn register_commands(http: &Arc<Http>, locales: &LocaleData) {
    for descriptor in inventory::iter::<CommandDescriptor>() {
        // Only top-level descriptors produce CreateCommand registrations.
        // Subcommands/groups are embedded by their parent's build_fn.
        if descriptor.kind == CommandKind::TopLevel {
            let command = (descriptor.build_fn)(locales);
            if descriptor.guild_only {
                // register as guild command
            } else {
                http.create_global_command(&command).await;
            }
        }
    }
}
```

## Dispatch via HashMap Instead of Match

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

static COMMAND_MAP: LazyLock<HashMap<&'static str, &'static CommandDescriptor>> =
    LazyLock::new(|| {
        inventory::iter::<CommandDescriptor>()
            .map(|d| (d.dispatch_name, d))
            .collect()
    });

pub async fn dispatch_command(
    ctx: &SerenityContext, command_interaction: &CommandInteraction,
) -> Result<()> {
    let (_kind, name) = guess_command_kind(command_interaction);

    let descriptor = COMMAND_MAP
        .get(name.as_str())
        .context(format!("Unknown command: {name}"))?;

    let start = Instant::now();
    (descriptor.handler_fn)(ctx.clone(), command_interaction.clone()).await?;

    // stats tracking unchanged
    let bot_data = ctx.data::<BotData>().clone();
    bot_data.increment_command_use_per_command(...).await;

    Ok(())
}
```

This replaces the entire 900-line match statement with ~15 lines. The `guess_command_kind` function still works the same
way — it flattens `/admin anilist add_anime_activity` into `"admin_anilist_add_anime_activity"` which is the HashMap
key.

## Before/After Comparison

### A simple command (`/anime`)

**Before (JSON + match):**

- `json/command/anime.json` (~50 lines of JSON with locale data)
- `command_dispatch.rs` match arm (~5 lines)
- `anilist_user/anime.rs` struct + `impl_command!` (~30 lines)
- Total: ~85 lines across 3 files

**After (convention macro):**

```rust
#[command]
pub async fn anime(ctx: CommandContext, #[autocomplete] anime_name: String) -> Result<EmbedsContents> {
    // handler logic (same as before, minus the struct boilerplate)
}
```

- Plus a few Fluent keys in `.ftl` files (which mostly already exist)
- Total: 1 attribute + function. No JSON file. No match arm.

### A subcommand group (`/admin anilist add_anime_activity`)

**Before:** `json/subcommand_group/admin.json` = 705 lines of JSON
**After:** ~30 lines of Rust with nested modules (see Level 3 example above)

## User Commands and Message Commands

These also get their own attribute:

```rust
#[user_command]
pub async fn avatar(ctx: CommandContext) -> Result<EmbedsContents> { ... }

#[message_command]
pub async fn translate(ctx: CommandContext) -> Result<EmbedsContents> { ... }
```

## Guild-Specific Commands

```rust
#[command(guild_only = true)]
pub async fn kill_switch(ctx: CommandContext) -> Result<EmbedsContents> { ... }
```

## Migration Strategy

Phased approach — both systems coexist during migration:

1. **Phase 1 — Build the proc macro crate** (`kasuki-macros`). Implement `#[command]`, `#[command_group]`,
   `#[subcommand_group]`, `#[subcommand]`. Add Fluent key convention for registration metadata.

2. **Phase 2 — Replace dispatch** with the `HashMap` approach. The HashMap dispatches to both old-style (struct-based)
   and new-style (function-based) handlers. Both coexist.

3. **Phase 3 — Migrate commands incrementally**. Start with a small group like `bot/` (3 commands). Each converted
   command removes its JSON file and match arm. Validate against Discord that registration still works.

4. **Phase 4 — Migrate subcommand groups**. Convert `admin` (the only 3-level group). This is the most complex but also
   the biggest win (705 lines of JSON eliminated).

5. **Phase 5 — Remove JSON infrastructure**. Delete `json/`, `register/structure/`, old registration functions.

## Crate Dependencies to Add

- `inventory` or `linkme` — zero-cost distributed static registration
- `syn`, `quote`, `proc-macro2` — for the proc macro crate (build-only dependency)

## Alternatives Considered

| Approach                              | Pros                                                                    | Cons                                                        |
|---------------------------------------|-------------------------------------------------------------------------|-------------------------------------------------------------|
| **Proc macro + inventory** (proposed) | Single source of truth, compile-time checked, zero boilerplate dispatch | Proc macro complexity, learning curve                       |
| **Build script code generation**      | No proc macro dependency, can still use JSON/TOML                       | Still two sources of truth, build step complexity           |
| **Trait object registry** (manual)    | No macro magic, explicit registration                                   | Still need manual `register()` calls somewhere, boilerplate |
| **Keep JSON but generate dispatch**   | Minimal change, auto-generate the match statement                       | JSON still disconnected from handler code                   |

## What This Gets You

- **Minimal boilerplate** — most commands need just `#[command]` and a function signature
- **All 3 Discord levels supported** — command, subcommand, subcommand group via module nesting
- **No match statement** — dispatch is automatic via HashMap
- **Compile-time checks** — invalid command definitions fail to compile
- **Localization stays in `.ftl` files** — single source of truth, no duplication
- **Easy to add commands** — write a function, add Fluent keys, done
- **Easy to add languages** — add a new `translation/{locale}/` folder, no Rust changes
