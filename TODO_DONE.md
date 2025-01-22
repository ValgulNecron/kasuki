- Anilist module:
    - [X] Finish comparison function.
    - [X] Add affinity score to user comparaison.
    - [X] Add character search function.
    - [X] Add staff search function.
    - [X] Add search feature with a type.
    - [X] Bind anilist account to discord for /user.
    - [X] Random /random {anime, manga}.
    - [X] Add a "delay" option to delay notification.
    - [X] Take [https://anilist.co/forum/thread/64835](https://anilist.co/forum/thread/64835) idea of generating image
      with a seiyuu and va role.
    - [X] Get all the register users of the server.
    - [X] Add studio search.
    - [X] Add commands that give the best waifu.
    - [X] Add caching to all requests.
    - [X] Send anime release to a channel.
    - [X] List all activity.
    - [X] Delete an activity.

- Anime module:
    - [X] Command for a random anime image.
    - [X] Command for a random anime image NSFW.

- Games module:
    - [ ] get game info from different platform (ubi (api not found), steam, epic(api not found), ea(api not found),
      etc…)
      Get the currency and language from the server language setting.
        - [X] Steam

- Audio module:
    - [X] ytdl play back 

- AI module:
    - [X] Image generation with AI.
    - [X] Video transcription.
    - [X] Video translation.
    - [X] Ask a question and reply the response.

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
        - [X] Rework the database to be more efficient.
    - [ ] Cache
    - [X] In memory cache.
        - [X] Create a parser because some description uses html and not markdown.
        - [X] Check [https://anilist.co/forum/thread/6125](https://anilist.co/forum/thread/6125) to be sure all cases
          are
          supported open issue here for markdown (https://github.com/ValgulNecron/markdown_converter)
    - [X] Figure out the necessary deps to work. Once found, change the dockerfile to use a debian base image to reduce
      size.
    - [X] Better error handling.
    - [X] Doing something with error else than logging it.
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