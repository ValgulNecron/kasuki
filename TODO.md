- Anilist module:
    - [X] Finish comparison function.
    - [X] Add affinity score to user comparaison.
    - [X] Add character search function.
    - [X] Add staff search function.
    - [X] Add search feature with a type.
    - [X] Bind anilist account to discord for /user.
    - [X] Random /random {anime, manga}.
    - [ ] Rework the xp in level.rs to something easier. — Too lazy to balance.
    - [X] Add caching to all requests.
    - [X] Send anime release to a channel.
    - [X] List all activity.
    - [X] Delete an activity.
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

- Anime module:
    - [X] Command for a random anime image.
    - [X] Command for a random anime image NSFW.

- Games module:
    - [ ] get game info from different platform (ubi (api not found), steam, epic(api not found), ea(api not found),
      etc…)
      Get the currency and language from the server language setting.
        - [X] Steam
        - [ ] Epic
        - [ ] Ubisoft
        - [ ] EA
        - [ ] GOG
        - [ ] Xbox
        - [ ] Playstation
    - [ ] get player stat
    - [ ] get free promotion notification
        - [ ] Steam
        - [ ] Epic
    - [ ] get game release notification
    - [ ] get game update notification

- AI submodule:
    - [X] Image generation with AI.
    - [X] Video transcription.
    - [X] Video translation.
    - [X] Ask a question and reply the response.

- Audio module: (Not sure if I will do this one. since it broke tos (in some case))
    - [ ] Play music from file upload.

- General module:
    - [X] Command to change the bot response language.
    - [X] Command to turn on and off module.
    - [X] Credit command.
    - [X] Bot info command.
    - [X] Ping command.
    - [X] Avatar command.
    - [X] Banner command.
    - [X] Command to recreate the server image from the guild member.
    - [X] Command to recreate the server image from member off all guild the bot is on.
    - [X] Get guild info.
    - [X] Profile command.
    - [X] Upgrade get guild info to show more info.
    - [X] Make the user profile better.


- General part:
    - [X] Add localisation.
        - [X] Localisation for response.
        - [X] Localisation for command.
    - [X] Find a name for the bot.
    - [X] Database
        - [x] Add a sqlite database.
        - [X] Add postgres database choices.
        - [ ] Rework the database to be more efficient.
    - [X] Cache
        - [ ] Add redis for cache.
        - [X] In memory cache.
    - [X] Create a parser because some description uses html and not markdown.
        - [X] Check [https://anilist.co/forum/thread/6125](https://anilist.co/forum/thread/6125) to be sure all cases
          are
          supported open issue here for markdown (https://github.com/ValgulNecron/markdown_converter)
    - [X] Figure out the necessary deps to work. Once found, change the dockerfile to use a debian base image to reduce
      size.
    - [X] Better error handling.
    - [X] Doing something with error else than logging it.
    - [ ] Rename function and variable for better clarity.
    - [ ] Add docs to every public function.
      (Run, Register and Autocomplete don’t need this.).
    - [X] Logging
    - [X] Updating to serenity 0.12.
    - [X] Support for command in dm.
    - [X] Make an anilist forum post.
    - [X] Rework the command registration to support all type of command.
        - [X] Command
        - [X] Guild Command
        - [X] Subcommand
        - [X] Subcommand group
        - [X] User command
        - [X] Message command (none exist but registration exist)
        - [X] User installed app.

- Optimisation needed:
    - [ ] anilist_server list_user

- Fix needed:
    - [ ] steam game search