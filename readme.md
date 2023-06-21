# TODO

### Website

for those of you who don't know rust. \
[https://github.com/ValgulNecron/Website_DiscordAnilistBot](https://github.com/ValgulNecron/Website_DiscordAnilistBot)

### BOT

- [ ] clean the code.
- [ ] finish comparison function.
- [ ] add character search function.
- [ ] add staff search function. - In Progress
- [ ] add search feature with type.
- [ ] find a name for the bot.
- [ ] take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) idea of generating image with
  seiyuu and va role.
- [x] add a bdd for some stuff prob sqllite but not sure. Added SQLLite db.
    - [ ] bind anilist account to discord for /user and /search user.
    - [ ] send anime release to a channel.
    - [ ] try to do the same for manga
      with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263) (for
      this one only selected manga not all seasonal)
    - [ ] activity command (auto send activity of a user to a channel).
    - [ ] add a "delay" option to delay notification. (like 1h for a translation)
    - [ ] add caching to all request. - In Progress, Added caching for random.
- [ ] random /random {anime, manga} - In Progress, Command and Cache here some error but do not respond. 
- [ ] when everything else is finished change how the text is display to support localisation.

# How to use

### 1. Docker.

- Install docker and docker compose.
- Clone this repo.

```bash
git clone https://github.com/ValgulNecron/DIscordAnilistBotRS.git
```

- edit compose-default.yml file and add your discord bot token. (not sure if it works or need to be renamed to
  compose.yml or docker-compose.yml)
- run docker compose.

```bash
docker compose up -d
```

Please remember that after a pull you will need to rebuild

```bash
docker compose up -d --build 
```

### 2. Rust.

- Install rust.
- Clone this repo.

```bash
git clone https://github.com/ValgulNecron/DIscordAnilistBotRS.git
```

- edit .env-default file and add your discord bot token and rename it to .env.
- run cargo.

```bash
cargo run --release
```

# Commands

- /info - show info about bot.
- /level - show your level based on what you read and watched.
- /user - show info about user.
- /anime - show info about anime.
- /manga - show info about manga.
- /ln - show info about light novel.
