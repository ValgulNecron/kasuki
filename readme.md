# TODO

### Website

for those of you who prefer web dev.. \
[https://github.com/ValgulNecron/kasuki_website](https://github.com/ValgulNecron/kasuki_website)

### BOT

- General part:
    - [ ] Change how the text is display to support localisation. - In progress. Top priority.
    - [X] Clean the code. - Done.
    - [X] Find a name for the bot. Found one kasuki. if you have any better don't hesitate to recommend.
    - [x] Add a bdd for some stuff prob sqlite but not sure. Added sqlite db.

- Anime submodule:
    - [X] Finish comparison function. - V1 done.
    - [X] Add character search function. Added character research with name.
    - [X] Add staff search function. Added staff research with name.
    - [X] Add search feature with type. Work for all.
    - [ ] Take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) idea of generating image
      with seiyuu and va role. - This is possible, I'm not competent enough.
    - [X] Bind anilist account to discord for /user and /search user. Added register command and edited user command.
    - [ ] Send anime release to a channel. - Will need some digging but should be possible.
    - [ ] Try to do the same for manga
      with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263) (for
      this one only selected manga not all seasonal). - Not sure if possible.
    - [ ] Activity command (auto send activity of a user to a channel). - This should be possible.
    - [ ] Add a "delay" option to delay notification. (like 1h for a translation). - Need anime notification first.
    - [ ] Add caching to all request. - In Progress, Added caching for random.
    - [X] Random /random {anime, manga}. Added random for both anime and manga. Manga random can give ln.

- AI submodule:
    - [X] Image generation with AI. - Done
    - [X] Video transcription. - Done
    - [X] Video translation. - Done
    - [ ] Ask a question and reply the response. - Not a priority

# Vision

The bot is in the first place a bot that interface the anilist api with discord letting user get different information
from it. there is also secondary module that will be added when I have idea or want to test thing.

# Contributing.

### I know how to code in rust.

them please check the todo and follow CONTRIBUTING.md to add feature, if the todo is complete or you want to do
something
else just do it.

### I don't know how to code in rust but still want to contribute.

1. You can add new langage by adding a translation in the file located in lang_file and add in lang.rs found in  
   src/cmd/general_modules/ the option with .add_string_choice("ISO-639-1", "ISO-639-1")
   please google for what is ISO-639-1
2. Contribute to this guide by making it clearer on how to use/ how it works.
3. Or by opening an issue with enhancement or new feature you want to see.

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
    - /help - Give a list of all command.
    - /info - Show info about bot.
    - /ping - Check if the bot respond to command.
    - /lang - let you change the langage for your guild. require admin perm.
- Anime:
    - /anime - Show info about anime.
    - /character - Show info on a character.
    - /compare - Compare 2 different user.
    - /level - Show your level based on what you read and watched.
    - /ln - Show info about light novel.
    - /manga - Show info about manga.
    - /random - Give a random anime or manga.
    - /register - Link your anilist and discord account.
    - /search - Let you search for different type. Like ln, manga, etc...
    - /staff - Give information about a specified staff.
    - /user - Show info about user.
- AI:
    - /image - Generate an image from a description.
    - /transcript - Transcript a video or an audio file with a size limit of 25mb.
    - /translation - Create a translated transcript of video or an audio file with a size limit of 25mb.

# Credit

- Thanks Srayeals for the badge I use as the bot pfp. (
  [https://anilist.co/forum/thread/20292](https://anilist.co/forum/thread/20292), [https://anilist.co/user/Srayeals](https://anilist.co/user/Srayeals))
- Annie May for the idea of having a discord bot linked to anilist (not the only one but the one I used and do not work
  anymore.)
- [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) For seiyuu and va role image generation
  idea.
