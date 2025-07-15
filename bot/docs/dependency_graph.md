# Kasuki Dependency Graph Documentation

This document provides a comprehensive overview of the dependencies between different components of the Kasuki Discord
bot. Understanding these relationships is crucial for development, maintenance, and troubleshooting.

## Core Components and Their Dependencies

### 1. Application Entry Point (`main.rs`)

The main entry point initializes all core components and establishes the foundation for the bot's operation.

**Dependencies:**

- **Configuration** (`config.rs`): Loads and manages bot configuration
- **Logging** (`logger.rs`): Initializes logging system
- **Database**: Initializes database connection
- **Discord Client**: Sets up Serenity client with appropriate intents
- **Event Handler**: Registers the event handler for Discord events
- **Songbird**: Initializes voice client for audio functionality
- **Background Tasks**: Indirectly launches background tasks via the event handler

```d2
Main: main.rs
Config: config.rs
Logger: logger.rs
DB: Database Connection
Discord: Discord Client
EventHandler: event_handler.rs
Songbird: Songbird Voice Client
BackgroundTasks: Background Tasks

Main -> Config
Main -> Logger
Main -> DB
Main -> Discord
Main -> EventHandler
Main -> Songbird
EventHandler -> BackgroundTasks
```

### 2. Event Handler (`event_handler.rs`)

The event handler processes Discord events and routes them to appropriate handlers.

**Dependencies:**

- **Command Dispatcher**: Routes command interactions to command handlers
- **Autocomplete Dispatcher**: Handles autocomplete interactions
- **Component Dispatcher**: Processes component interactions (buttons, select menus)
- **Background Task Launcher**: Initializes background tasks on bot ready
- **Database**: Stores user and guild data from events
- **Bot Data**: Shared state including caches and configuration

```d2
EventHandler: event_handler.rs
CommandDispatch: command_dispatch.rs
AutocompleteDispatch: autocomplete_dispatch.rs
ComponentsDispatch: components_dispatch.rs
BackgroundLauncher: background_launcher.rs
Database: Database
BotData: Shared Bot Data

EventHandler -> CommandDispatch
EventHandler -> AutocompleteDispatch
EventHandler -> ComponentsDispatch
EventHandler -> BackgroundLauncher
EventHandler -> Database
EventHandler -> BotData
```

### 3. Command System

The command system handles user interactions through Discord slash commands.

**Dependencies:**

- **Command Dispatcher**: Routes commands to appropriate handlers
- **Command Handlers**: Implement command functionality
- **Database**: Retrieves and stores data for commands
- **External APIs**: Interfaces with Anilist, VNDB, etc.
- **Caches**: Stores frequently accessed data

```d2
CommandDispatch: command_dispatch.rs
CommandHandlers: Command Handlers
Database: Database
ExternalAPIs: External APIs
Caches: Caches
Helper: Helper Utilities

CommandDispatch -> CommandHandlers
CommandHandlers -> Database
CommandHandlers -> ExternalAPIs
CommandHandlers -> Caches
CommandHandlers -> Helper
```

### 4. Background Tasks (`background_task` module)

Background tasks perform periodic operations independent of user interactions.

**Dependencies:**

- **Task Launcher**: Orchestrates and schedules all background tasks
- **Database**: Retrieves and stores data for background operations
- **External APIs**: Fetches data from external sources
- **Caches**: Updates cached data
- **Discord API**: Interacts with Discord for status updates, etc.

```d2
BackgroundLauncher: background_launcher.rs
AnisongUpdater: get_anisong_db.rs
RandomStatsUpdater: update_random_stats.rs
ActivityManager: anime_activity.rs
GameManager: game_management
PingManager: ping_manager
BotInfoUpdater: bot_info_updater
BlacklistUpdater: user_blacklist
ServerImageManager: server_image
Database: Database
AnilistCache: Anilist Cache
SteamAPI: Steam API
GitHub: GitHub Repository

BackgroundLauncher -> AnisongUpdater
BackgroundLauncher -> RandomStatsUpdater
BackgroundLauncher -> ActivityManager
BackgroundLauncher -> GameManager
BackgroundLauncher -> PingManager
BackgroundLauncher -> BotInfoUpdater
BackgroundLauncher -> BlacklistUpdater
BackgroundLauncher -> ServerImageManager

AnisongUpdater -> Database
RandomStatsUpdater -> AnilistCache
ActivityManager -> Database
ActivityManager -> AnilistCache
GameManager -> SteamAPI
PingManager -> Database
ServerImageManager -> Database
BlacklistUpdater -> GitHub
```

### 5. Database System

The database system manages persistent storage of bot data.

**Entities:**

- **ActivityData**: Anime/manga activity tracking
- **AnimeSong**: Anime song database
- **GuildData**: Discord server information
- **GuildLang**: Server language preferences
- **GuildSubscription**: Server premium subscriptions
- **KillSwitch**: Emergency feature disabling
- **ModuleActivation**: Module enabling/disabling per server
- **PingHistory**: Bot latency tracking
- **RegisteredUser**: Users registered with the bot
- **ServerImage**: Server image generation data
- **ServerUserRelation**: Server-user relationships
- **UserColor**: User color preferences
- **UserData**: User information
- **UserSubscription**: User premium subscriptions

```d2
Database: Database
ActivityData: ActivityData
AnimeSong: AnimeSong
GuildData: GuildData
GuildLang: GuildLang
GuildSubscription: GuildSubscription
KillSwitch: KillSwitch
ModuleActivation: ModuleActivation
PingHistory: PingHistory
RegisteredUser: RegisteredUser
ServerImage: ServerImage
ServerUserRelation: ServerUserRelation
UserColor: UserColor
UserData: UserData
UserSubscription: UserSubscription

Database -> ActivityData
Database -> AnimeSong
Database -> GuildData
Database -> GuildLang
Database -> GuildSubscription
Database -> KillSwitch
Database -> ModuleActivation
Database -> PingHistory
Database -> RegisteredUser
Database -> ServerImage
Database -> ServerUserRelation
Database -> UserColor
Database -> UserData
Database -> UserSubscription
```

### 6. External API Integrations

The bot integrates with several external APIs to provide functionality.

**Integrations:**

- **Anilist API**: Anime and manga data
- **VNDB API**: Visual novel data
- **Steam API**: Game information
- **Discord API**: Core bot functionality
- **Lavalink**: Music playback

```d2
ExternalAPIs: External APIs
AnilistAPI: Anilist GraphQL API
VNDBAPI: VNDB API
SteamAPI: Steam API
DiscordAPI: Discord API
LavalinkAPI: Lavalink API

ExternalAPIs -> AnilistAPI
ExternalAPIs -> VNDBAPI
ExternalAPIs -> SteamAPI
ExternalAPIs -> DiscordAPI
ExternalAPIs -> LavalinkAPI
```

## Cross-Cutting Concerns

### 1. Configuration System

The configuration system manages bot settings and is used by almost all components.

**Consumers:**

- **Main**: Initial setup
- **Commands**: Feature toggles and limits
- **Background Tasks**: Task intervals and behavior
- **Database**: Connection information
- **External APIs**: API keys and endpoints

### 2. Caching System

The caching system improves performance by storing frequently accessed data.

**Cache Types:**

- **Anilist Cache**: Anime and manga data
- **VNDB Cache**: Visual novel data
- **Command Usage Cache**: Command usage statistics

**Consumers:**

- **Commands**: Retrieve cached data
- **Background Tasks**: Update cached data

### 3. Error Handling

Error handling is implemented throughout the application for robustness.

**Key Features:**

- **Error Propagation**: Errors are propagated up the call stack
- **Error Logging**: Errors are logged for debugging
- **User Feedback**: User-friendly error messages are sent to Discord

## Initialization Flow

The following diagram illustrates the initialization flow of the bot:

```d2
direction: right

Main: main.rs
Config: Configuration
DB: Database
Discord: Discord Client
EventHandler: Event Handler
BackgroundTasks: Background Tasks

Main -> Config: "Load configuration"
Main -> DB: "Initialize database"
Main -> Discord: "Create Discord client"
Main -> EventHandler: "Register event handler"
Discord -> EventHandler: "Trigger ready event"
EventHandler -> BackgroundTasks: "Launch background tasks"
```

## Data Flow

The following diagram illustrates the data flow for a typical command execution:

```d2
direction: right

User: Discord User
Discord: Discord API
EventHandler: Event Handler
CommandDispatch: Command Dispatcher
CommandHandler: Command Handler
ExternalAPI: External API
DB: Database
Cache: Cache

User -> Discord: "Execute slash command"
Discord -> EventHandler: "Interaction create event"
EventHandler -> CommandDispatch: "Dispatch command"
CommandDispatch -> CommandHandler: "Execute command"
CommandHandler -> Cache: "Check for cached data"
Cache -> CommandHandler: "Return cached data (if exists)" {
  style.stroke-dash: 5
}
CommandHandler -> ExternalAPI: "Request data (if needed)"
ExternalAPI -> CommandHandler: "Return data" {
  style.stroke-dash: 5
}
CommandHandler -> DB: "Store/retrieve data"
DB -> CommandHandler: "Return data" {
  style.stroke-dash: 5
}
CommandHandler -> Cache: "Update cache"
CommandHandler -> Discord: "Send response" {
  style.stroke-dash: 5
}
Discord -> User: "Display response" {
  style.stroke-dash: 5
}
```

## Module Dependencies

This section details the dependencies between different modules in the codebase.

### Command Module Dependencies

Commands depend on various components to function:

- **User Commands**: Depend on UserData entity
- **Anilist Commands**: Depend on Anilist API, RegisteredUser entity, and Anilist cache
- **Music Commands**: Depend on Songbird and Lavalink
- **Admin Commands**: Depend on GuildData, ModuleActivation entities
- **Server Commands**: Depend on GuildData, ServerImage entities

### Background Task Dependencies

Background tasks have specific dependencies:

- **Anisong Updater**: Depends on AnimeSong entity and external API
- **Activity Manager**: Depends on ActivityData, RegisteredUser entities and Anilist API
- **Server Image Manager**: Depends on ServerImage, UserColor entities
- **Ping Manager**: Depends on PingHistory entity
- **Game Manager**: Depends on Steam API

## Conclusion

This dependency graph documentation provides a comprehensive overview of the relationships between different components
of the Kasuki Discord bot. Understanding these dependencies is crucial for maintaining and extending the bot's
functionality.

When making changes to the codebase, consider the impact on dependent components and ensure that all dependencies are
properly managed.
