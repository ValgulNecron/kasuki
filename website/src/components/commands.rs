use leptos::prelude::*;

#[component]
pub fn Commands() -> impl IntoView {
    let (active_tab, set_active_tab) = signal("anime");

    let tabs = vec!["anime", "anime_nsfw", "vn", "ai", "minigame", "music", "steam", "user", "levels", "bot", "server"];

    view! {
        <section class="section commands" id="commands">
            <div class="container">
                <div class="section-title">
                    <h2>"Command Examples"</h2>
                    <p>"Explore Kasuki's wide range of commands organized by category."</p>
                </div>
                <div class="command-tabs" role="tablist" aria-label="Command categories">
                    {tabs.into_iter()
                        .map(|tab| {
                            let tab_name = tab.replace("_", " ");
                            let aria_label = format!("Show {} commands", tab_name);
                            view! {
                                <button
                                    class="command-tab"
                                    class:active=move || active_tab.get() == tab
                                    on:click=move |_| set_active_tab.set(tab)
                                    role="tab"
                                    aria-selected=move || if active_tab.get() == tab { "true" } else { "false" }
                                    aria-label=aria_label.clone()
                                >
                                    {tab_name}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <div class="command-content">
                    <div class="command-group anime" class:active=move || active_tab.get() == "anime" data-tab="anime">
                        <div class="command">
                            <h3><span class="command-name">"/anilist_user"</span>"Get AniList User Information"</h3>
                            <p>"Get detailed information about an AniList user, including their anime and manga stats, favorites, and activity."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/anime"</span>"Get Anime Information"</h3>
                            <p>"Search for an anime and get detailed information including synopsis, genres, studios, and ratings."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/character"</span>"Get Character Information"</h3>
                            <p>"Search for anime and manga characters and view their details, background, and appearances."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/compare"</span>"Compare Two AniList Users"</h3>
                            <p>"Compare two AniList users and see their anime/manga taste compatibility, shared favorites, and rating differences."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/ln"</span>"Get Light Novel Information"</h3>
                            <p>"Search for light novels and get detailed information including synopsis, genres, and authors."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/manga"</span>"Get Manga Information"</h3>
                            <p>"Search for manga and get detailed information including synopsis, genres, authors, and publication status."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/random"</span>"Get Random Anime Content"</h3>
                            <p>"Discover random anime, manga, or characters when you're looking for something new to explore."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/seiyuu"</span>"Get Voice Actor Information"</h3>
                            <p>"Get an image of a voice actor (seiyuu) with 4 of their notable character roles for easy reference."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/staff"</span>"Get Staff Information"</h3>
                            <p>"Look up information about anime and manga industry staff members, including directors, writers, and artists."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/studio"</span>"Get Studio Information"</h3>
                            <p>"Get information about anime studios, including their production history and notable works."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/waifu"</span>"Get Waifu Information"</h3>
                            <p>"Discover and share information about popular anime waifus and character favorites."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/list_activity"</span>"View AniList Activity"</h3>
                            <p>"Check recent activity from AniList users, including updates to their anime and manga lists."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/list_user"</span>"List Registered Users"</h3>
                            <p>"View a list of users who have registered their AniList accounts with the bot."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/register"</span>"Register AniList Username"</h3>
                            <p>"Register your AniList username for ease of use with other anime-related commands."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/level"</span>"Get User Level"</h3>
                            <p>"Check the level of an AniList user based on their anime and manga activity."</p>
                        </div>
                    </div>
                    <div class="command-group anime_nsfw" class:active=move || active_tab.get() == "anime_nsfw" data-tab="anime_nsfw">
                        <div class="command">
                            <h3><span class="command-name">"/random_hanime random_himage"</span>"Get Random NSFW Anime Image"</h3>
                            <p>"Get a random NSFW anime image of various types (waifu, neko, trap)."</p>
                        </div>
                    </div>
                    <div class="command-group vn" class:active=move || active_tab.get() == "vn" data-tab="vn">
                        <div class="command">
                            <h3><span class="command-name">"/vn character"</span>"Get VN Character Information"</h3>
                            <p>"Get info of a character from a visual novel."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/vn game"</span>"Get Visual Novel Information"</h3>
                            <p>"Get detailed information about a visual novel game."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/vn producer"</span>"Get VN Producer Information"</h3>
                            <p>"Get info of a producer from a visual novel."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/vn staff"</span>"Get VN Staff Information"</h3>
                            <p>"Get info of a staff member from a visual novel."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/vn user"</span>"Get VN Database User Information"</h3>
                            <p>"Get info of a user from a visual novel database."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/vn stats"</span>"Get VN API Stats"</h3>
                            <p>"Get stats of the visual novel API."</p>
                        </div>
                    </div>
                    <div class="command-group ai" class:active=move || active_tab.get() == "ai" data-tab="ai">
                        <div class="command">
                            <h3><span class="command-name">"/ai image"</span>"Generate AI Images"</h3>
                            <p>"Generate custom images using AI based on your text descriptions."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/ai question"</span>"Ask AI Questions"</h3>
                            <p>"Ask questions and get AI-generated answers (the AI has no conversation context)."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/ai transcript"</span>"Generate Video Transcript"</h3>
                            <p>"Generate a transcript from a video file."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/ai translation"</span>"Generate Translation"</h3>
                            <p>"Generate a translation for a video file."</p>
                        </div>
                    </div>
                    <div class="command-group minigame" class:active=move || active_tab.get() == "minigame" data-tab="minigame">
                        <div class="command">
                            <h3><span class="command-name">"/minigame fishing"</span>"Go Fishing"</h3>
                            <p>"Go fishing to catch random fish."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/minigame inventory"</span>"View Inventory"</h3>
                            <p>"View your inventory of items."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/minigame fish_inventory"</span>"View Fish Inventory"</h3>
                            <p>"View a detailed inventory of all your fish with rarity information."</p>
                        </div>
                    </div>
                    <div class="command-group music" class:active=move || active_tab.get() == "music" data-tab="music">
                        <div class="command">
                            <h3><span class="command-name">"/music play"</span>"Play Music"</h3>
                            <p>"Play a song from a search query or URL."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music pause"</span>"Pause Music"</h3>
                            <p>"Pause the current playback."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music resume"</span>"Resume Music"</h3>
                            <p>"Resume the paused playback."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music stop"</span>"Stop Music"</h3>
                            <p>"Stop the current playback."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music skip"</span>"Skip Track"</h3>
                            <p>"Skip to the next track in the queue."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music queue"</span>"View Queue"</h3>
                            <p>"Display the current playlist queue."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music clear"</span>"Clear Queue"</h3>
                            <p>"Clear the playlist queue."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music remove"</span>"Remove Track"</h3>
                            <p>"Remove a specific track from the queue."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music seek"</span>"Seek Position"</h3>
                            <p>"Seek to a specific time in the current track."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music swap"</span>"Swap Tracks"</h3>
                            <p>"Swap the positions of two tracks in the queue."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music join"</span>"Join Voice Channel"</h3>
                            <p>"Make the bot join your voice channel."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/music leave"</span>"Leave Voice Channel"</h3>
                            <p>"Make the bot leave the voice channel."</p>
                        </div>
                    </div>
                    <div class="command-group steam" class:active=move || active_tab.get() == "steam" data-tab="steam">
                        <div class="command">
                            <h3><span class="command-name">"/steam game"</span>"Get Steam Game Information"</h3>
                            <p>"Get info of a steam game including details, pricing, and requirements."</p>
                        </div>
                    </div>
                    <div class="command-group user" class:active=move || active_tab.get() == "user" data-tab="user">
                        <div class="command">
                            <h3><span class="command-name">"/user avatar"</span>"Get User Avatar"</h3>
                            <p>"Get the avatar of a Discord user."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/user banner"</span>"Get User Banner"</h3>
                            <p>"Get the banner of a Discord user."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/user profile"</span>"View User Profile"</h3>
                            <p>"Show the profile of a Discord user with detailed information."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/user command_usage"</span>"View Command Usage"</h3>
                            <p>"Show the usage statistics of each command for a user."</p>
                        </div>
                    </div>
                    <div class="command-group levels" class:active=move || active_tab.get() == "levels" data-tab="levels">
                        <div class="command">
                            <h3><span class="command-name">"/levels stats"</span>"View User Statistics"</h3>
                            <p>"Get the user statistics including activity levels and rankings."</p>
                        </div>
                    </div>
                    <div class="command-group bot" class:active=move || active_tab.get() == "bot" data-tab="bot">
                        <div class="command">
                            <h3><span class="command-name">"/bot credit"</span>"View Bot Credits"</h3>
                            <p>"Get the credit information of the app and its developers."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/bot info"</span>"View Bot Information"</h3>
                            <p>"Get detailed information about the bot, including version and features."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/bot ping"</span>"Check Bot Ping"</h3>
                            <p>"Get the ping of the bot (and the shard id) to check response time."</p>
                        </div>
                    </div>
                    <div class="command-group server" class:active=move || active_tab.get() == "server" data-tab="server">
                        <div class="command">
                            <h3><span class="command-name">"/server guild"</span>"View Guild Information"</h3>
                            <p>"Get detailed information about the Discord guild/server."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/server guild_image"</span>"Generate Guild Image"</h3>
                            <p>"Generate an image using the guild server image and the user profile picture."</p>
                        </div>
                        <div class="command">
                            <h3><span class="command-name">"/server guild_image_g"</span>"Generate Global Guild Image"</h3>
                            <p>"Generate an image using the guild server image and the global profile picture cache."</p>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}