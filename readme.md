[![Build & Publish Release Docker Image](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-release.yml/badge.svg?branch=master)](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-release.yml)
[![Build & Publish Release Dev Image](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-dev.yml/badge.svg?branch=dev)](https://github.com/ValgulNecron/kasuki/actions/workflows/docker-image-dev.yml)
[![Rust Clippy](https://github.com/ValgulNecron/kasuki/actions/workflows/linting.yml/badge.svg?branch=master)](https://github.com/ValgulNecron/kasuki/actions/workflows/linting.yml)
[![Rust Testing](https://github.com/ValgulNecron/kasuki/actions/workflows/testing.yml/badge.svg)](https://github.com/ValgulNecron/kasuki/actions/workflows/testing.yml)
![Code Activity](https://img.shields.io/github/commit-activity/w/valgulnecron/kasuki/master?style=plastic)
![Dev Code Activity](https://img.shields.io/github/commit-activity/w/valgulnecron/kasuki/dev?style=plastic&label=Dev)

![Alt](https://repobeats.axiom.co/api/embed/ce0c4fc4155948704332a4126e892cfe612ed6cb.svg "Repobeats analytics image")

![Alt](https://repobeats.axiom.co/api/embed/ce0c4fc4155948704332a4126e892cfe612ed6cb.svg "Repobeats analytics image")

# Vision

The bot is in the first place,
a bot that interfaces discord and the anilist api,
letting users get different information from it.
There are also multiple secondary modules that will be added
when I have ideas or want to test things.

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

You can if you want use a postgres database and not a sqlite one the user will need to be able to create a database
(cache and data).
Create table inside both database
and select, insert, delete on them.

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
Please also add the country code to the constant LANG_MAP in src/constant.rs and add the langage in full name with it.

for the command json please follow the example,
the "code" field should respect discord locale https://discord.com/developers/docs/reference#locales

# TODO

## BOT

Check the [todo](TODO.md) file.

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
