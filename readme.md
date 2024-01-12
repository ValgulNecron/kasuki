[![Build & Publish Release Docker Image](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-release.yml/badge.svg?branch=master)](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-release.yml)
[![Build & Publish Release Dev Image](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-dev.yml/badge.svg?branch=dev)](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-dev.yml)
[![Rust Clippy](https://github.com/ValgulNecron/kasuki/actions/workflows/linting.yml/badge.svg?branch=master)](https://github.com/ValgulNecron/kasuki/actions/workflows/linting.yml)
[![Rust Testing](https://github.com/ValgulNecron/kasuki/actions/workflows/testing.yml/badge.svg)](https://github.com/ValgulNecron/kasuki/actions/workflows/testing.yml)
![Code Activity](https://img.shields.io/github/commit-activity/w/valgulnecron/kasuki/master?style=plastic)
![Dev Code Activity](https://img.shields.io/github/commit-activity/w/valgulnecron/kasuki/dev?style=plastic&label=Dev)


# Vision


The bot is in the first place,
a bot that interfaces discord and the anilist api,
letting users get different information from it.
There are also multiple secondary modules that will be added
when I have ideas or want to test things.


# Contributing


## I know how to code in rust


Then please check the todo and follow CONTRIBUTING.md to add feature if the todo is complete, or you want to do
something else, you can do it and open a pr afterward.


## I don’t know how to code in rust but still want to contribute

1. You can add a new language by adding a translation in the file located in json and adding it to the choices in
   json/command/lang.json
2. Contribute to this guide by making it clearer on how to use/ how it works.
3. Or by opening an issue with enhancement or new feature you want to see.
4. Or by contributing to the website for the bot.

Please note that for embed you will need to use the country code.
please also add the country code to the constant LANG_MAP in src/constant.rs and add the langage in full name with it.

for the command json please follow the example,
the "code" field should respect discord locale https://discord.com/developers/docs/reference#locales


# How to use


## 1. Add the bot to your server


you can add my instance of the bot
with [this link](https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot)


## 2. Self-host your instance


### tested on:


linux: ubuntu 22.04.2 x86-64

Requirement: libssl-dev libsqlite3-dev libpng-dev libjpeg-dev ca-certificates

windows: windows 10 and 11


### Docker

- Install docker and docker compose.
- Clone this repo.

```bash
git clone https://github.com/ValgulNecron/DIscordAnilistBotRS.git
```

- edit compose-default.yml file and add your discord bot token and edit the other env var.
  (not sure if it works or needs to be renamed to
  compose.yml or docker-compose.yml)
- run docker compose.

```bash
docker compose up -d
```

```bash
docker compose up -d --pull always
```

or you can build from the latest commit.


### or Rust

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

```bash
cargo build --release
```

# TODO


## BOT

- General part:
  - [X] Change how the text is displayed to support localization.
  - [X] Find a name for the bot.
  - [x] Add a database for some stuff.
  - [ ] Add more database choices like postgres and more when I have time.
  - [X] Banner. Show your or a specified user banner.
  - [X] Profil. Show a user profile and some info.
  - [X] Avatar. show you the profile picture of a user.
  - [X] Add support to turn on and off module.
  - [X] Create a parser because some description uses html and not markdown.
  - [ ] Check [https://anilist.co/forum/thread/6125](https://anilist.co/forum/thread/6125) to be sure all cases are
    supported
  - [ ] Poll feature with custom choice and a graph afterward for comparison.
  - [X] Figure out the necessary deps to work. Once found, change the dockerfile to use a debian base image to reduce
    size.
  - [X] Better error handling.
  - [ ] Doing something with error else than logging it.
  - [X] Localisation for response.
  - [X] Localisation for command.
  - [X] Rename function, structure, command name etc... so it makes more sense.
  - [ ] Add docs to every public function.
    (Run, Register and Autocomplete don’t need this.).
  - [X] Logging
  - [X] Updating to serenity 0.12.
  - [X] Support for command in dm.
  - [X] Make an anilist forum post.

- Anime submodule:
  - [X] Finish comparison function.
  - [X] Add affinity score to user comparaison.
  - [X] Add character search function.
  - [X] Add staff search function.
  - [X] Add search feature with a type.
  - [X] Bind anilist account to discord for /user.
  - [X] Random /random {anime, manga}.
  - [ ] Rework the xp in struct_level to something easier. — Too lazy to balance.
  - [X] Add caching to all requests.
  - [X] Send anime release to a channel.
  - [X] List all activity.
  - [ ] Delete an activity.
  - [ ] Try to do the same for manga.
    with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263).
    — Did some digging seem possible.
  - [ ] Activity command (auto sends activity of a user to a channel).
    — Same as anime, but this one will be hard since
    a user can do update every second like every year. Will either have delay or be resource intensive.
  - [X] Add a "delay" option to delay notification.
  - [X] Take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) idea of generating image
    with a seiyuu and va role.
  - [X] Get all the register users of the server.
  - [X] Add studio search.
  - [X] Add commands that give the best waifu.

- AI submodule:
  - [X] Image generation with AI.
  - [X] Video transcription.
  - [X] Video translation.
  - [ ] Ask a question and reply the response.
    — Not a priority.

- Games module:
  - [X] get game info from different platform (ubi (api not found), steam, epic(api not found), ea(api not found), etc....)
    Get the currency and language from the server language setting.
    Steam done.
  - [ ] get player stat
  - [ ] get free promotion notification

## Website


for those of you who prefer web dev.\
[https://github.com/ValgulNecron/kasuki_website](https://github.com/ValgulNecron/kasuki_website)


# Credit

- Thanks Srayeals for the badge I use as the bot
  pfp. ([https://anilist.co/forum/thread/20292](https://anilist.co/forum/thread/20292), [https://anilist.co/user/Srayeals](https://anilist.co/user/Srayeals)) \
  for the bot
  pfp [https://anilist.co/forum/thread/20292/comment/2206321](https://anilist.co/forum/thread/20292/comment/2206321) [Valedstorm Olivia](https://i.imgur.com/vERcUNo.png) \
  for the beta
  bot [https://srayealsbadges.carrd.co/#h](https://srayealsbadges.carrd.co/#h) [Neptune](https://srayealsbadges.carrd.co/assets/images/gallery77/7846fb0b_original.png?v=0ff4ab06) \
  for the dev bot (private only on my
  server) [https://srayealsbadges.carrd.co/#s](https://srayealsbadges.carrd.co/#s) [Krul tepes](https://srayealsbadges.carrd.co/assets/images/gallery121/67449fb5_original.png?v=0ff4ab06)
- Annie May for the idea of having a discord bot linked to anilist (not the only one but the one I used and do not work
  anymore. (and now it seems to work again.))
- [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) For seiyuu and va role image generation
  idea.
  seem like the post was removed.
- [https://github.com/Skittyblock/AniBot](https://github.com/Skittyblock/AniBot) For auto-complete on command, did not
  even know it existed before.

# Stat


![kasuki](https://counter.valgul.moe/get/@kasuki?theme=gelbooru)
