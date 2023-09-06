## TODO

### BOT

- General part:
  - [X] Change how the text is displayed to support localization. — Done.
  - [X] Clean the code. — Done.
  - [X] Find a name for the bot. — Found one kasuki. if you have any better, don't hesitate to recommend.
  - [x] Add a bdd for some stuff prob sqlite but not sure. — Added sqlite db.
  - [X] Banner. Show your or a specified user banner. — Done.
  - [X] Profil. Show a user profile and some info. — Done.
  - [X] Add support to turn on and off module. — Done.
  - [X] Create a parser because some desc uses html and not markdown. — Done. Will need to check to be sure all is
    done.
  - [ ] Poll feature with custom choice and a graph afterward for comparison.
  - [X] Figure out the necessary deps to work. Once found, change the dockerfile to use an ubuntu base image to reduce
    size — Done.
  - [X] Better error handling.
    Different error messages,
    type and replies everytime not in certain condition.
    — working on it.
    Should be olay believe there is still some risk of panic will need to see in the long term.
  - [ ] Localisation for response — Done except for some other "minor" stuff.
  - [ ] Localisation for command — Working on it. Compare will be done later.
  - [ ] Rename function, structure, command name etc... so it makes more sense.

- Anime submodule:
  - [X] Finish comparison function.   
    — V1 done. — Will need to be better, but it works.
    Ideas of improvement add a score on
    how close two people are (affinity score).
    — And have better formatting for the text.
  - [X] Add character search function. — Added character research with name.
  - [X] Add staff search function. — Added staff research with name.
  - [X] Add search feature with a type. — Work for all.
  - [X] Bind anilist account to discord for /user. — Added register command and edited user command.
  - [X] Random /random {anime, manga}. — Added random for both anime and manga. Manga random can give ln.
  - [ ] Rework the xp in struct_level to something easier. — Too lazy to balance
  - [X] Add caching to all requests. — Done now will need to rework random.rs cause it double cache. 3days cache
    except
    for "high" priority request like user data.
  - [X] Send anime release to a channel.
    — Done
    - [ ] List all activity
    - [ ] Delete an activity.
  - [ ] Try to do the same for manga.
    with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263) 
    (for
    this one only selected manga not all seasonal).
    — Did some digging seem possible. I will do anime-first trough.
  - [ ] Activity command (auto sends activity of a user to a channel).
    — Same as anime, but this one will be hard since
    a user can do update every second like every year. Will either have delay or be resource intensive.
  - [X] Add a "delay" option to delay notification.
    — (like 1h for a translation).
    — Need anime notification first.
    it will be easy once anime is done. On wait but will be done
  - [ ] Take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) 
    idea of generating image
    with a seiyuu and va role.
    — This is possible, I'm not competent enough.
  - [ ] Get all the register users of the server.
    Working on it after finishing anime activity.
  - [X] Add studio search.
  - [X] Add commands that give the best waifu. — Done.

- AI submodule:
  - [X] Image generation with AI. — Done.
  - [X] Video transcription. — Done.
  - [X] Video translation. — Done.
  - [ ] Ask a question and reply the response. — Not a priority.

### Website


for those of you who prefer web dev.\
[https://github.com/ValgulNecron/kasuki_website](https://github.com/ValgulNecron/kasuki_website)


## Vision


The bot is in the first place,
a bot that interfaces discord and the anilist api, 
letting users get different information from it.
There are also multiple secondary modules that will be added
when I have ideas or want to test things.


## Contributing


### I know how to code in rust


Then please check the todo and follow CONTRIBUTING.md to add feature if the todo is complete, or you want to do
something else, just do it and open a pr afterward.


### I don't know how to code in rust but still want to contribute

1. You can add new langage by adding a translation in the file located in lang_file and adding it to
   lang_file/available_lang.json
2. Contribute to this guide by making it clearer on how to use/ how it works.
3. Or by opening an issue with enhancement or new feature you want to see.

Please note that for embed you will need to use ISO-639-1 and if no ISO-639-1 exist or need to be more specific like the
different "version" of chinese use ISO-639-3 and if still not like with a specific chinese written in pinyin, I used the
ISO-639-3 code for mandarin chinese (cmn) and added a p for pinyin

if working on command_register please use the same structure,
but the "code" field should respect discord locale https://discord.com/developers/docs/reference#locales


## How to use


### 1. Add the bot to your server


you can add my instance of the bot
with [this link](https://discord.com/api/oauth2/authorize?client_id=923286536445894697&permissions=533113194560&scope=bot)


### 2. Self-host your instance


##### tested on:


linux: ubuntu x86-64

Requirement: libssl-dev libsqlite3-dev libpng-dev libjpeg-dev ca-certificates

windows: windows 10 and 11


##### Docker

- Install docker and docker compose.
- Clone this repo.

```bash
git clone https://github.com/ValgulNecron/DIscordAnilistBotRS.git
```

- edit compose-default.yml file and add your discord bot token.
  (not sure if it works or needs to be renamed to
  compose.yml or docker-compose.yml)
- run docker compose.

```bash
docker compose up -d
```

Please remember that after a pull you will need to rebuild

```bash
docker compose up -d --build 
```

##### or Rust

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

## Commands


## /!\ Not updated /!\

- General:
  - /info - Show info about bot.
  - /ping - Check if the bot responds to command.
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
  - /search - Let you search for a different type. Like ln, manga, etc...
  - /staff - Give information about a specified staff.
  - /user - Show info about user.
- AI:
  - /image - Generate an image from a description.
  - /transcript - Transcript a video or an audio file with a size limit of 25mb.
  - /translation 
  — Create a translated transcript of video or an audio file with a size limit of 25mb.

## Credit

- Thanks Srayeals for the badge I use as the bot
  pfp. ([https://anilist.co/forum/thread/20292](https://anilist.co/forum/thread/20292), [https://anilist.co/user/Srayeals](https://anilist.co/user/Srayeals))
- Annie May for the idea of having a discord bot linked to anilist (not the only one but the one I used and do not work
  anymore.)
- [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) For seiyuu and va role image generation
  idea.
- [https://github.com/Skittyblock/AniBot](https://github.com/Skittyblock/AniBot) For auto-complete on command, did not
  even know it existed before.

## Stat

![kasuki](https://counter.valgul.moe/get/@kasuki?theme=gelbooru)
[![Codacy Badge](https://app.codacy.com/project/badge/Grade/987dc84319a24235a346a662ca9045c1)](https://app.codacy.com/gh/ValgulNecron/kasuki/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)
![Code Activity](https://img.shields.io/github/commit-activity/w/valgulnecron/kasuki/master?style=plastic)
