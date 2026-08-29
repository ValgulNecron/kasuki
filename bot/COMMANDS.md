# Kasuki Bot Commands

## Parent Command Groups

| Parent          | Description                                                |
|-----------------|------------------------------------------------------------|
| `user`          | General purpose commands for user                          |
| `ai`            | Command from the AI module                                 |
| `bot`           | Command to get information about the bot                   |
| `music`         | Command from the Music module                              |
| `steam`         | Steam command from the GAME module                         |
| `server`        | General purpose commands for server                        |
| `vn`            | Get info of a VN                                           |
| `random_anime`  | Command from the ANIME module                              |
| `random_hanime` | Command from the ANIME module (NSFW)                       |
| `levels`        | Command to get level of user, and statistic                |
| `minigame`      | Commands for playing minigames and managing your inventory |
| `admin`         | Bot configuration for admin only                           |

---

## AniList Commands (top-level)

| Command                           | Description                       | File                                |
|-----------------------------------|-----------------------------------|-------------------------------------|
| `/anime <anime_name>`             | Info of an anime                  | `command/anilist_user/anime.rs`     |
| `/manga <manga_name>`             | Info of a manga                   | `command/anilist_user/manga.rs`     |
| `/ln <ln_name>`                   | Info of a light novel             | `command/anilist_user/ln.rs`        |
| `/character <name>`               | Info of a character               | `command/anilist_user/character.rs` |
| `/anilist_user [username]`        | Info of an user on AniList        | `command/anilist_user/user.rs`      |
| `/compare <username> <username2>` | Compare 2 users                   | `command/anilist_user/compare.rs`   |
| `/level [username]`               | Get the level of a user           | `command/anilist_user/level.rs`     |
| `/random <type>`                  | Get a random anime or manga       | `command/anilist_user/random.rs`    |
| `/register <username>`            | Register your username on AniList | `command/anilist_user/register.rs`  |
| `/seiyuu <staff_name>`            | Info of a seiyuu                  | `command/anilist_user/seiyuu.rs`    |
| `/staff <staff_name>`             | Info of a staff                   | `command/anilist_user/staff.rs`     |
| `/studio <studio>`                | Info of a studio                  | `command/anilist_user/studio.rs`    |
| `/waifu`                          | Get a random waifu                | `command/anilist_user/waifu.rs`     |

## AniList Server Commands (top-level)

| Command          | Description                         | File                                           |
|------------------|-------------------------------------|------------------------------------------------|
| `/list_activity` | Get the list of registered activity | `command/anilist_server/list_all_activity.rs`  |
| `/list_user`     | Get the list of registered users    | `command/anilist_server/list_register_user.rs` |

## User Commands (`/user`)

| Command                          | Description                               | File                            |
|----------------------------------|-------------------------------------------|---------------------------------|
| `/user avatar [username]`        | Get the avatar                            | `command/user/avatar.rs`        |
| `/user banner [username]`        | Get the banner                            | `command/user/banner.rs`        |
| `/user command_usage [username]` | Show the usage of each command for a user | `command/user/command_usage.rs` |
| `/user profile [username]`       | Show the profile of a user                | `command/user/profile.rs`       |

## AI Commands (`/ai`)

| Command           | Description                                      | File                        |
|-------------------|--------------------------------------------------|-----------------------------|
| `/ai image`       | Generate an image                                | `command/ai/image.rs`       |
| `/ai question`    | Ask a question and get the response (no context) | `command/ai/question.rs`    |
| `/ai translation` | Generate a translation                           | `command/ai/translation.rs` |
| `/ai transcript`  | Generate a transcript from a video               | `command/ai/transcript.rs`  |

## Bot Commands (`/bot`)

| Command       | Description                                | File                    |
|---------------|--------------------------------------------|-------------------------|
| `/bot credit` | Get the credit of the app                  | `command/bot/credit.rs` |
| `/bot info`   | Get information on the bot                 | `command/bot/info.rs`   |
| `/bot ping`   | Get the ping of the bot (and the shard id) | `command/bot/ping.rs`   |

## Music Commands (`/music`)

| Command                 | Description                            | File                      |
|-------------------------|----------------------------------------|---------------------------|
| `/music clear`          | Clear the current queue                | `command/music/clear.rs`  |
| `/music join`           | Join the voice channel                 | `command/music/join.rs`   |
| `/music leave`          | Leave the voice channel                | `command/music/leave.rs`  |
| `/music pause`          | Pause the current song                 | `command/music/pause.rs`  |
| `/music play <search>`  | Play a song                            | `command/music/play.rs`   |
| `/music queue`          | Show the current queue                 | `command/music/queue.rs`  |
| `/music remove <index>` | Remove a song from the queue           | `command/music/remove.rs` |
| `/music resume`         | Resume the current song                | `command/music/resume.rs` |
| `/music seek <time>`    | Seek to a position in the current song | `command/music/seek.rs`   |
| `/music skip`           | Skip the current song                  | `command/music/skip.rs`   |
| `/music stop`           | Stop the current song                  | `command/music/stop.rs`   |
| `/music swap <i1> <i2>` | Swap two songs in the queue            | `command/music/swap.rs`   |

## Steam Commands (`/steam`)

| Command                   | Description              | File                               |
|---------------------------|--------------------------|------------------------------------|
| `/steam game <game_name>` | Get info of a steam game | `command/steam/steam_game_info.rs` |

## Server Commands (`/server`)

| Command                 | Description                                   | File                              |
|-------------------------|-----------------------------------------------|-----------------------------------|
| `/server guild`         | Get info of the guild                         | `command/server/guild.rs`         |
| `/server guild_image`   | Generate profile picture for the guild        | `command/server/guild_image.rs`   |
| `/server guild_image_g` | Generate global profile picture for the guild | `command/server/guild_image_g.rs` |

## Visual Novel Commands (`/vn`)

| Command                | Description                   | File                      |
|------------------------|-------------------------------|---------------------------|
| `/vn character <name>` | Get info of a VN character    | `command/vn/character.rs` |
| `/vn game <title>`     | Get info of a visual novel    | `command/vn/game.rs`      |
| `/vn producer <name>`  | Get info of a VN producer     | `command/vn/producer.rs`  |
| `/vn staff <name>`     | Get info of a VN staff member | `command/vn/staff.rs`     |
| `/vn stats`            | Get VN statistics             | `command/vn/stats.rs`     |
| `/vn user <username>`  | Get info of a VN user         | `command/vn/user.rs`      |

## Anime Image Commands

| Command                               | Description                          | File                                  |
|---------------------------------------|--------------------------------------|---------------------------------------|
| `/random_anime random_image <type>`   | Get a random anime image             | `command/anime/random_image.rs`       |
| `/random_hanime random_himage <type>` | Get a random nsfw anime image (NSFW) | `command/anime_nsfw/random_himage.rs` |

## Levels Commands (`/levels`)

| Command         | Description         | File                      |
|-----------------|---------------------|---------------------------|
| `/levels stats` | Get stats for level | `command/levels/stats.rs` |

## Minigame Commands (`/minigame`)

| Command                    | Description               | File                                 |
|----------------------------|---------------------------|--------------------------------------|
| `/minigame fishing`        | Go fishing!               | `command/minigame/fishing.rs`        |
| `/minigame fish_inventory` | Check your fish inventory | `command/minigame/fish_inventory.rs` |
| `/minigame inventory`      | Check your inventory      | `command/minigame/inventory.rs`      |
| `/minigame trivia`         | Trivia minigame           | `command/minigame/trivia.rs`         |

## Admin Commands (`/admin`)

### General (`/admin general`)

| Command                             | Description                               | File                             |
|-------------------------------------|-------------------------------------------|----------------------------------|
| `/admin general lang <lang_choice>` | Change the language of the bot's response | `command/admin/server/lang.rs`   |
| `/admin general module <name>`      | Turn on or off a module                   | `command/admin/server/module.rs` |

### AniList (`/admin anilist`)

| Command                                              | Description              | File                                       |
|------------------------------------------------------|--------------------------|--------------------------------------------|
| `/admin anilist add_anime_activity <anime> [delays]` | Add an anime activity    | `command/admin/anilist/add_activity.rs`    |
| `/admin anilist delete_anime_activity <anime>`       | Delete an anime activity | `command/admin/anilist/delete_activity.rs` |

## Management Commands (owner-only, restricted to specific guild)

| Command                                   | Description                              | File                                     |
|-------------------------------------------|------------------------------------------|------------------------------------------|
| `/kill_switch <name> <state>`             | Globally turn on or off a module         | `command/management/kill_switch.rs`      |
| `/give_premium_sub <user> <subscription>` | Give a premium subscription to a user    | `command/management/give_premium_sub.rs` |
| `/remove_test_sub <user>`                 | Remove premium subscriptions from a user | `command/management/remove_test_sub.rs`  |

---

**Total: 67 commands across 12 parent groups**

All command source files are in `bot/src/command/`. Parent command definitions are in `bot/src/command/parents.rs`. Registry and dispatch logic are in `bot/src/command/registry.rs` and `bot/src/command/command_dispatch.rs`.
