- General part:
    - [X] Change how the text is displayed to support localization.
    - [X] Find a name for the bot.
    - [x] Add a sqlite database.
    - [X] Add postgres database choices.
    - [ ] Add more database choices when I have time.
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
    - [X] Doing something with error else than logging it.
    - [X] Localisation for response.
    - [X] Localisation for command.
    - [ ] Rename for better clarity.
    - [ ] Add docs to every public function.
      (Run, Register and Autocomplete don’t need this.).
    - [X] Logging
    - [X] Updating to serenity 0.12.
    - [X] Support for command in dm.
    - [X] Make an anilist forum post.
    - [ ] Rework the command registration to support sub command group and remove line duplicate.

- Anime submodule:
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

- AI submodule:
    - [X] Image generation with AI.
    - [X] Video transcription.
    - [X] Video translation.
    - [X] Ask a question and reply the response.
- Games module :
    - [ ] get game info from different platform (ubi (api not found), steam, epic(api not found), ea(api not found),
      etc…)
      Get the currency and language from the server language setting.
    - [X] Search for a steam game.
    - [ ] get player stat
    - [ ] get free promotion notification