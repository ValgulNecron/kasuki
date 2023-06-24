# TODO

### Website

for those of you who don't know rust. \
[https://github.com/ValgulNecron/Website_DiscordAnilistBot](https://github.com/ValgulNecron/Website_DiscordAnilistBot)

### BOT

- Anime submodule
  - [ ] Clean the code.
  - [ ] Finish comparison function.
  - [ ] Add character search function.
  - [X] Add staff search function. Added staff research with name.
  - [ ] Add search feature with type.
  - [ ] Find a name for the bot. - In progress have a temporary one kazuki. (will need to find a better one)
  - [ ] Take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) idea of generating image with
   seiyuu and va role.
  - [x] Add a bdd for some stuff prob sqllite but not sure. Added SQLLite db.
  - [ ] Bind anilist account to discord for /user and /search user. 
  - [ ] Send anime release to a channel.
  - [ ] Try to do the same for manga
    with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263) (for
    this one only selected manga not all seasonal)
  - [ ] Activity command (auto send activity of a user to a channel).
  - [ ] Add a "delay" option to delay notification. (like 1h for a translation)
  - [ ] Add caching to all request. - In Progress, Added caching for random.
  - [X] Random /random {anime, manga}. Added random for both anime and manga. Manga random can give ln.
  - [ ] When everything else is finished change how the text is display to support localisation.

- AI submodule.
    - [X] Image generation with ai.
    - [ ] Video transcription.
    - [ ] Video translation.

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

- General:
  - /info - Show info about bot. 
  - /ping - Check if the bot respond to command.  
  - /help - Give a list of all command.
- Anime:
  - /anime - Show info about anime.
  - /compare - Compare 2 different user.
  - /level - Show your level based on what you read and watched.
  - /ln - Show info about light novel.
  - /manga - Show info about manga.
  - /random - Give a random anime or manga.
  - /staff - Give information about a specified staff.
  - /user - Show info about user.
- AI:
  - /image - Generate an image from a description.

# Credit

- [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) For seiyuu and va role image generation idea.
- Annie May for the idea of having a discord bot linked to anilist (not the only one but the one I used and do not work anymore.)