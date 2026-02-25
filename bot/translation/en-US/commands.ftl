# Command registration translations for en-US
# Used by registration_dispatcher.rs for Discord command localization

# ─── Parent commands ─────────────────────────────────────────────────────────

parent-user-name = general
parent-user-desc = General purpose commands.

parent-ai-name = ai
parent-ai-desc = Command from the AI module

parent-bot-name = bot
parent-bot-desc = Command to get information about the bot.

parent-music-name = music
parent-music-desc = Command from the Music module

parent-steam-name = steam
parent-steam-desc = Steam command from the GAME module.

parent-server-name = server
parent-server-desc = General purpose commands for server.

parent-vn-desc = Get info of a visual novel

parent-random_anime-name = random_anime
parent-random_anime-desc = Command from the ANIME module

parent-random_hanime-name = random_hanime
parent-random_hanime-desc = Command from the ANIME module

parent-levels-name = levels
parent-levels-desc = Command to get level of user, and statistic.

parent-minigame-name = minigame
parent-minigame-desc = Commands for playing minigames and managing your inventory.

parent-admin-name = admin
parent-admin-desc = Bot configuration configuration for admin only.

# ─── Subcommand groups ────────────────────────────────────────────────────────

group-admin-anilist-name = anilist
group-admin-anilist-desc = Commands for the AniList module that need admin permissions.

group-admin-general-name = general
group-admin-general-desc = Commands for the general module that need admin permissions.

# ─── Commands ─────────────────────────────────────────────────────────────────

# admin/anilist
cmd-add_anime_activity-name = add_anime_activity
cmd-add_anime_activity-desc = Add an anime activity.

cmd-delete_anime_activity-name = delete_anime_activity
cmd-delete_anime_activity-desc = Delete an anime activity.

# admin/general
cmd-lang-name = lang
cmd-lang-desc = The language you want to set the response to.

cmd-module-name = module
cmd-module-desc = Turn on or off a module.

# ai
cmd-image-name = image
cmd-image-desc = Generate an image.

cmd-question-name = question
cmd-question-desc = Ask a question and get the response (this is not a chat it has no context).

cmd-transcript-name = transcript
cmd-transcript-desc = Generate a transcript from a video.

cmd-translation-name = translation
cmd-translation-desc = Generate a translation.

# anilist_server
cmd-list_activity-name = list_activity
cmd-list_activity-desc = Get the list of registered activity.

cmd-list_user-name = list_user
cmd-list_user-desc = Get the list of registered user.

# anilist_user
cmd-anime-name = anime
cmd-anime-desc = Info of an anime.

cmd-character-name = character
cmd-character-desc = Info of a character.

cmd-compare-name = compare
cmd-compare-desc = Compare 2 user.

cmd-level-name = level
cmd-level-desc = Get the level of a user.

cmd-ln-name = ln
cmd-ln-desc = Info of a light novel.

cmd-manga-name = manga
cmd-manga-desc = Info of a manga.

cmd-random-name = random
cmd-random-desc = Get a random anime or manga.

cmd-register-name = register
cmd-register-desc = Register your username on AniList.

cmd-seiyuu-name = seiyuu
cmd-seiyuu-desc = Info of a seiyuu.

cmd-staff-name = staff
cmd-staff-desc = Info of a staff.

cmd-studio-name = studio
cmd-studio-desc = Info of a studio.

cmd-anilist_user-name = anilist_user
cmd-anilist_user-desc = Info of an user on AniList.

cmd-waifu-name = waifu
cmd-waifu-desc = Get a random waifu.

# anime
cmd-random_image-name = random_image
cmd-random_image-desc = Get a random anime image.

# anime_nsfw
cmd-random_himage-name = random_himage
cmd-random_himage-desc = Get a random nsfw anime image.

# bot
cmd-credit-name = credit
cmd-credit-desc = Get the credit of the app.

cmd-info-name = info
cmd-info-desc = Get information on the bot.

cmd-ping-name = ping
cmd-ping-desc = Get the ping of the bot (and the shard id).

# levels
cmd-stats-name = stats
cmd-stats-desc = Get stats for level.

# management
cmd-give_premium_sub-name = give_premium_sub
cmd-give_premium_sub-desc = Give a premium subscription to a user.

cmd-kill_switch-name = kill_switch
cmd-kill_switch-desc = Globally turn on or off a module

cmd-remove_test_sub-name = remove_test_sub
cmd-remove_test_sub-desc = Remove premium subscriptions from a user.

# minigame
cmd-fish_inventory-name = fish_inventory
cmd-fish_inventory-desc = Check your fish inventory.

cmd-fishing-name = fishing
cmd-fishing-desc = Go fishing!

cmd-inventory-name = inventory
cmd-inventory-desc = Check your inventory.

# music
cmd-clear-name = clear
cmd-clear-desc = Clear the current queue.

cmd-join-name = join
cmd-join-desc = Join the voice channel.

cmd-leave-name = leave
cmd-leave-desc = Leave the voice channel.

cmd-pause-name = pause
cmd-pause-desc = Pause the current song.

cmd-play-name = play
cmd-play-desc = Play a song.

cmd-queue-name = queue
cmd-queue-desc = Show the current queue.

cmd-remove-name = remove
cmd-remove-desc = Remove a song from the queue.

cmd-resume-name = resume
cmd-resume-desc = Resume the current song.

cmd-seek-name = seek
cmd-seek-desc = Seek to a position in the current song.

cmd-skip-name = skip
cmd-skip-desc = Skip the current song.

cmd-stop-name = stop
cmd-stop-desc = Stop the current song.

cmd-swap-name = swap
cmd-swap-desc = Swap two songs in the queue.

# server
cmd-guild_image-name = guild_image
cmd-guild_image-desc = Generate profile picture for the guild.

cmd-guild_image_g-name = guild_image_g
cmd-guild_image_g-desc = Generate global profile picture for the guild.

cmd-guild-name = guild
cmd-guild-desc = Get info of the guild.

# steam
cmd-game-name = game
cmd-game-desc = Get info of a steam game.

# user
cmd-avatar-name = avatar
cmd-avatar-desc = Get the avatar.

cmd-banner-name = banner
cmd-banner-desc = Get the banner.

cmd-command_usage-name = command_usage
cmd-command_usage-desc = Show the usage of each command for an user.

cmd-profile-name = profile
cmd-profile-desc = Show the profile of a user.

# vn
cmd-vn_game-desc = Get info of a visual novel.

cmd-vn_character-desc = Get info of a VN character.

cmd-vn_staff-desc = Get info of a VN staff member.

cmd-vn_producer-desc = Get info of a VN producer.

cmd-vn_user-desc = Get info of a VN user.

cmd-vn_stats-desc = Get VN statistics.

# ─── Arg translations ─────────────────────────────────────────────────────────

# admin/anilist/add_anime_activity
arg-add_anime_activity-anime_name-name = anime_name
arg-add_anime_activity-anime_name-desc = Name of the anime you want to add as an activity.
arg-add_anime_activity-delays-name = delays
arg-add_anime_activity-delays-desc = A delay in seconds.

# admin/anilist/delete_anime_activity
arg-delete_anime_activity-anime_name-name = anime_name
arg-delete_anime_activity-anime_name-desc = Name of the anime you want to delete as an activity.

# admin/general/lang
arg-lang-lang_choice-name = lang_choice
arg-lang-lang_choice-desc = The language you want to set the response to.

# admin/general/module
arg-module-name-name = module_name
arg-module-name-desc = The module you want to change the state of.
arg-module-state-name = module_state
arg-module-state-desc = The state you want to to.

# ai/image
arg-image-description-name = description
arg-image-description-desc = Enter a description of the image you want to generate.
arg-image-n-name = n
arg-image-n-desc = Number of images to generate.

# ai/question
arg-question-prompt-name = prompt
arg-question-prompt-desc = What you want to ask.

# ai/transcript
arg-transcript-video-name = video
arg-transcript-video-desc = Upload video file (max. 25MB).
arg-transcript-prompt-name = prompt
arg-transcript-prompt-desc = A guide text for audio style. Must match the audio language.
arg-transcript-lang-name = lang
arg-transcript-lang-desc = Select input language (ISO-639-1)

# ai/translation
arg-translation-video-name = video
arg-translation-video-desc = Upload video file (max. 25MB).
arg-translation-lang-name = lang
arg-translation-lang-desc = Select input language (ISO-639-1)

# anilist_user/anime
arg-anime-anime_name-name = anime_name
arg-anime-anime_name-desc = Name of the anime you want to check.

# anilist_user/character
arg-character-name-name = name
arg-character-name-desc = Name of the character you want to check.

# anilist_user/compare
arg-compare-username-name = username
arg-compare-username-desc = Username of the first user you want to compare.
arg-compare-username2-name = username2
arg-compare-username2-desc = Username of the second user you want to compare.

# anilist_user/level
arg-level-username-name = username
arg-level-username-desc = Username of the user you want the level of.

# anilist_user/ln
arg-ln-ln_name-name = ln_name
arg-ln-ln_name-desc = Name of the light novel you want to check.

# anilist_user/manga
arg-manga-manga_name-name = manga_name
arg-manga-manga_name-desc = Name of the manga you want to check.

# anilist_user/random
arg-random-type-name = type
arg-random-type-desc = Type of the random (anime or manga).

# anilist_user/register
arg-register-username-name = username
arg-register-username-desc = Username you want to register.

# anilist_user/seiyuu
arg-seiyuu-staff_name-name = seiyuu_name
arg-seiyuu-staff_name-desc = Name of the seiyuu you want to check.

# anilist_user/staff
arg-staff-staff_name-name = staff_name
arg-staff-staff_name-desc = Name of the staff you want to check.

# anilist_user/studio
arg-studio-studio-name = studio
arg-studio-studio-desc = Name of the studio you want to check.

# anilist_user/user
arg-anilist_user-username-name = username
arg-anilist_user-username-desc = Username of the user you want to check.

# anime/random_image
arg-random_image-image_type-name = image_type
arg-random_image-image_type-desc = Type of the image you want.

# anime_nsfw/random_himage
arg-random_himage-image_type-name = image_type
arg-random_himage-image_type-desc = Type of the image you want.

# management/give_premium_sub
arg-give_premium_sub-user-name = user
arg-give_premium_sub-user-desc = The user to give the subscription to.
arg-give_premium_sub-subscription-name = subscription
arg-give_premium_sub-subscription-desc = The subscription to give.

# management/kill_switch
arg-kill_switch-name-name = module_name
arg-kill_switch-name-desc = The module you want to change the state of.
arg-kill_switch-state-name = module_state
arg-kill_switch-state-desc = The state you want to to.

# management/remove_test_sub
arg-remove_test_sub-user-name = user
arg-remove_test_sub-user-desc = The user to remove the subscription from.

# music/play
arg-play-search-name = search
arg-play-search-desc = Search for a song.

# music/remove
arg-remove-index-name = index
arg-remove-index-desc = Index of the song to remove.

# music/seek
arg-seek-time-name = time
arg-seek-time-desc = Time to seek to in seconds.

# music/swap
arg-swap-index1-name = index1
arg-swap-index1-desc = Index of the first song.
arg-swap-index2-name = index2
arg-swap-index2-desc = Index of the second song.

# steam/game
arg-game-game_name-name = game_name
arg-game-game_name-desc = Name of the steam game you want info of.

# user/avatar
arg-avatar-username-name = username
arg-avatar-username-desc = Username of the user you want the avatar of.

# user/banner
arg-banner-username-name = username
arg-banner-username-desc = Username of the user you want the avatar of.

# user/command_usage
arg-command_usage-username-name = username
arg-command_usage-username-desc = Username of the user you want the usage of.

# user/profile
arg-profile-username-name = username
arg-profile-username-desc = Username of the user you want the avatar of.

# vn/game
arg-vn_game-title-name = title
arg-vn_game-title-desc = Title of the visual novel.

# vn/character
arg-vn_character-name-name = name
arg-vn_character-name-desc = Name of the character.

# vn/staff
arg-vn_staff-name-name = name
arg-vn_staff-name-desc = Name of the staff member.

# vn/producer
arg-vn_producer-name-name = name
arg-vn_producer-name-desc = Name of the producer.

# vn/user
arg-vn_user-username-name = username
arg-vn_user-username-desc = Username of the VN user.

# ─── Choice translations ──────────────────────────────────────────────────────

# admin/general/lang choices
choice-lang-lang_choice-en-name = English
choice-lang-lang_choice-jp-name = Japanese
choice-lang-lang_choice-de-name = German
choice-lang-lang_choice-fr-name = French
choice-lang-lang_choice-es-ES-name = Spanish
choice-lang-lang_choice-zh-CN-name = Chinese (Simplified)
choice-lang-lang_choice-ru-name = Russian

# admin/general/module choices
choice-module-name-AI-name = AI
choice-module-name-ANILIST-name = ANILIST
choice-module-name-GAME-name = Game
choice-module-name-ANIME-name = Anime
choice-module-name-VN-name = Visual Novel
choice-module-name-LEVEL-name = Level
choice-module-name-MINIGAME-name = Mini game
