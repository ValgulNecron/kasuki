# TODO 
- clean the code.
- finish comparison function.
- add character search function.
- add staff search function.
- add search feature with type.
- find a name for the bot.
- take https://anilist.co/forum/thread/64835 idea of generating image with seiyuu and va role.
- activity command (auto send activity of a user to a channel).
- add a bdd for some stuff prob sqllite but not sure.

# How to use

## 1. Docker. 
- Install docker and docker compose.
- Clone this repo. 
```bash
git clone https://github.com/ValgulNecron/DIscordAnilistBotRS.git
```
- edit compose-default.yml file and add your discord bot token. (not sure if it works or need to be renamed to compose.yml or docker-compose.yml)
- run docker compose.
```bash
docker compose up -d
```
## 2. Rust.
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
