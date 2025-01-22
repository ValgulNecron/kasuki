- Anilist module:
    - [ ] Rework the xp in level.rs to something easier. — Too lazy to balance.
    - [ ] Activity command for manga.
      with [https://www.mangaupdates.com/series.html?id=70263](https://www.mangaupdates.com/series.html?id=70263).
      — Did some digging seem possible.
    - [ ] Activity command (auto sends activity of a user to a channel).
      — Same as anime, but this one will be hard since
      a user can do update every second like every year. Will either have delay or be resource intensive.
    - [ ] Better compare command.


- Anime module:
    - [ ]

- Games module:
    - [ ] get game info from different platform (ubi (api not found), steam, epic(api not found), ea(api not found),
      etc…)
      Get the currency and language from the server language setting.
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

- AI module:
    - [ ] Make generic for api.
    - [ ] Rewrite the code to be cleaner.

- Audio module: (Not sure if I will do this one. since it broke tos (in some case))
    - [ ] Play music from file upload.

- General module:
    - [ ] Better langage json loading.

- General part:
    - [ ] Cache
        - [ ] Add redis for cache.
    - [ ] Rename function and variable for better clarity.
    - [ ] Add docs to every public function.
      (Run, Register, and Autocomplete don’t need this.).
    - [ ] Rework the bot way of working to be clearer.
    - [ ] Change image to alpine to reduce size.
    - [ ] switch to new command way.

- Optimisation needed:
    - [ ] anilist_server list_user

- Fix needed:
    - [ ] steam game search