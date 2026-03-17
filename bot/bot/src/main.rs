use crate::event_handler::{BotData, Handler};
use crate::logger::{create_log_directory, init_logger};
use anyhow::Context;
use shared::cache::CacheInterface;
use shared::config::{Config, DbConfig};
use shared::image_saver::storage::{create_image_store, ImageStore};

use serenity::all::GatewayIntents;
use serenity::cache::Settings as CacheSettings;
use serenity::secrets::Token;
use serenity::Client;
use songbird::driver::DecodeMode;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

pub mod autocomplete;
pub mod bot_data;
mod command;
mod components;
mod constant;
mod custom_serenity_impl;
pub mod error_management;
mod event_handler;
mod handlers;
mod helper;
pub mod launch_task;
mod logger;
mod music_events;
mod register;
mod server_image;
mod structure;

#[tokio::main]
async fn main() {
	if let Err(e) = run().await {
		error!("Fatal error: {:#}", e);
		process::exit(1);
	}
}

async fn run() -> anyhow::Result<()> {
	rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.expect("Failed to install default CryptoProvider");

	println!("Preparing bot environment please wait...");
	let config: Config = Config::new().context("Failed to read config.toml")?;
	println!("Config loaded successfully");

	let _sentry_guard = config.sentry_url.as_deref().map(|url| {
		let guard = sentry::init((
			url,
			sentry::ClientOptions {
				release: sentry::release_name!(),
				..Default::default()
			},
		));
		println!("Sentry initialized successfully");
		guard
	});

	let log = config.logging.log_level.clone();
	let max_log_retention_days = config.logging.max_log_retention;
	create_log_directory().context("Failed to create log directory")?;
	let _guard = init_logger(log.as_str(), max_log_retention_days)
		.context("Failed to initialize logger")?;
	info!("Logger initialized successfully with level: {}", log);
	info!("Log retention days: {}", max_log_retention_days);

	info!("Loading locales");
	shared::localization::load_locales().context("Failed to load locales")?;
	info!("Locales loaded successfully");

	let discord_token = config.bot.discord_token.clone();
	info!("Bot token length: {}", discord_token.len());

	info!("Initializing database");
	let db_config = config.db.clone();
	init_db(db_config)
		.await
		.context("Database initialization failed")?;
	info!("Database initialized successfully");

	let cache_config = config.cache.clone();
	info!("Initializing caches (backend: {})", cache_config.cache_type);
	let anilist_cache: Arc<CacheInterface> = Arc::new(
		match CacheInterface::from_config(&cache_config).await {
			Ok(c) => {
				info!("AniList cache initialized with {} backend", cache_config.cache_type);
				c
			},
			Err(e) => {
				warn!(
					"Failed to init AniList cache with {} backend, falling back to memory: {}",
					cache_config.cache_type, e
				);
				CacheInterface::new()
			},
		},
	);
	let vndb_cache: Arc<CacheInterface> = Arc::new(
		match CacheInterface::from_config(&cache_config).await {
			Ok(c) => {
				info!("VNDB cache initialized with {} backend", cache_config.cache_type);
				c
			},
			Err(e) => {
				warn!(
					"Failed to init VNDB cache with {} backend, falling back to memory: {}",
					cache_config.cache_type, e
				);
				CacheInterface::new()
			},
		},
	);
	info!("Caches initialized successfully");

	info!("Connecting to database");
	let connection = config
		.db
		.connect()
		.await
		.context("Failed to connect to the database")?;
	info!("Database connection established successfully");

	info!("Configuring Discord gateway intents");
	let gateway_intent_non_privileged =
		GatewayIntents::non_privileged() | GatewayIntents::GUILD_VOICE_STATES;
	info!(
		"Non-privileged intents configured: {:?}",
		gateway_intent_non_privileged
	);

	let gateway_intent_privileged = GatewayIntents::GUILD_MEMBERS;
	info!(
		"Privileged intents configured: {:?}",
		gateway_intent_privileged
	);

	let mut intent = gateway_intent_non_privileged;
	intent |= gateway_intent_privileged;
	let gateway_intent = intent;
	info!("Combined gateway intents: {:?}", gateway_intent);

	info!("Finished preparing the environment. Starting the bot.");

	info!("Parsing Discord token");
	let discord_token =
		Token::from_str(discord_token.as_str()).context("Failed to parse Discord token")?;
	info!("Discord token parsed successfully");

	info!("Initializing Songbird voice client");
	let songbird_config = songbird::Config::default().decode_mode(DecodeMode::Decode);
	info!(
		"Songbird configured with decode mode: {:?}",
		DecodeMode::Decode
	);

	let manager = songbird::Songbird::serenity_from_config(songbird_config);
	info!("Songbird voice client initialized successfully");

	let (shutdown_tx, _) = broadcast::channel(1);
	info!("Created shutdown signal channel");

	info!("Initializing image store (type: {})", config.image.storage.storage_type);
	let image_store: Arc<dyn ImageStore> = Arc::from(
		create_image_store(&config.image.storage).context("Failed to create image store")?,
	);
	info!("Image store initialized successfully");

	let (user_color_tx, user_color_rx) =
		tokio::sync::mpsc::unbounded_channel::<shared::queue::tasks::ImageTask>();
	let (server_image_tx, server_image_rx) =
		tokio::sync::mpsc::unbounded_channel::<shared::queue::tasks::ImageTask>();

	info!("Connecting to Redis for image queue");
	let redis_connection = {
		let queue_config = &config.queue;
		let redis_url = queue_config.redis_url();
		match redis::Client::open(redis_url.as_str()) {
			Ok(client) => match client.get_multiplexed_async_connection().await {
				Ok(conn) => {
					info!(
						"Connected to Redis at {}:{}",
						queue_config.host, queue_config.port
					);
					Arc::new(RwLock::new(Some(conn)))
				},
				Err(e) => {
					warn!(
						"Failed to connect to Redis (image queue will be unavailable): {}",
						e
					);
					Arc::new(RwLock::new(None))
				},
			},
			Err(e) => {
				warn!(
					"Failed to create Redis client (image queue will be unavailable): {}",
					e
				);
				Arc::new(RwLock::new(None))
			},
		}
	};

	info!("Initializing bot data structure");
	let bot_data: Arc<BotData> = Arc::new(BotData {
		config: Arc::new(config),
		bot_info: Arc::new(RwLock::new(None)),
		anilist_cache,
		vndb_cache,
		already_launched: false.into(),
		apps: Arc::new(Default::default()),
		user_blacklist: Arc::new(Default::default()),
		db_connection: Arc::new(connection),
		manager: Arc::clone(&manager),
		http_client: Arc::from(reqwest::Client::new()),
		shard_manager: Default::default(),
		lavalink: Arc::new(Default::default()),
		shutdown_signal: Arc::new(shutdown_tx),
		vocal_session: Arc::new(Default::default()),
		user_color_update_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
		server_image_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
		redis_connection,
		user_color_task_tx: user_color_tx,
		server_image_task_tx: server_image_tx,
		image_store,
	});
	info!("Bot data structure initialized successfully");

	let bd = bot_data.clone();
	tokio::spawn(
		crate::launch_task::queue_publisher::user_color_queue_publisher(user_color_rx, bd),
	);
	let bd = bot_data.clone();
	tokio::spawn(
		crate::launch_task::queue_publisher::server_image_queue_publisher(server_image_rx, bd),
	);
	info!("Queue publisher tasks spawned");

	info!("Creating Discord client");
	let mut cache_settings = CacheSettings::default();
	cache_settings.max_messages = 0;
	cache_settings.cache_guilds = true;
	cache_settings.cache_channels = true;
	cache_settings.cache_users = true;

	let mut client = Client::builder(discord_token, gateway_intent)
		.cache_settings(cache_settings)
		.data(bot_data.clone())
		.voice_manager(manager)
		.event_handler(Arc::new(Handler))
		.await
		.context("Failed to create Discord client")?;
	info!("Discord client created successfully");

	let shutdown_signal = bot_data.shutdown_signal.clone();
	info!("Starting Discord client with auto-sharding");
	tokio::spawn(async move {
		if let Err(why) = client.start_autosharded().await {
			error!("Discord client error: {:?}", why);
			let _ = shutdown_signal.send(());
		}

		info!("Discord client shutdown gracefully");
		drop(client);
	});

	#[cfg(unix)]
	{
		info!("Setting up signal handlers for Unix environment");
		let mut sigint =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
				.expect("failed to register SIGINT handler");
		info!("Registered SIGINT handler");

		let mut sigterm =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
				.expect("failed to register SIGTERM handler");
		info!("Registered SIGTERM handler");

		let mut sigquit =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit())
				.expect("failed to register SIGQUIT handler");
		info!("Registered SIGQUIT handler");

		let mut sigusr1 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1())
				.expect("failed to register SIGUSR1 handler");
		info!("Registered SIGUSR1 handler");

		let mut sigusr2 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined2())
				.expect("failed to register SIGUSR2 handler");
		info!("Registered SIGUSR2 handler");

		info!("All Unix signal handlers registered successfully, waiting for signals");

		let mut shutdown_rx = bot_data.shutdown_signal.subscribe();

		tokio::select! {
			_ = sigint.recv() => { info!("Received SIGINT signal"); },
			_ = sigterm.recv() => { info!("Received SIGTERM signal"); },
			_ = sigquit.recv() => { info!("Received SIGQUIT signal"); },
			_ = sigusr1.recv() => { info!("Received SIGUSR1 signal"); },
			_ = sigusr2.recv() => { info!("Received SIGUSR2 signal"); },
			_ = shutdown_rx.recv() => { info!("Received internal shutdown signal"); },
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		info!("Sending shutdown signal to all background tasks");
		if let Err(e) = bot_data.shutdown_signal.send(()) {
			warn!("Failed to send shutdown signal: {}", e);
		} else {
			info!("Shutdown signal sent successfully");
		}

		info!("Waiting for background tasks to shut down gracefully");
		tokio::time::sleep(Duration::from_secs(2)).await;
		info!("Proceeding with bot shutdown");
	}

	#[cfg(windows)]
	{
		info!("Setting up signal handlers for Windows environment");
		let mut ctrl_break = tokio::signal::windows::ctrl_break()
			.expect("failed to register CTRL+BREAK handler");
		info!("Registered CTRL+BREAK handler");

		let mut ctrl_c = tokio::signal::windows::ctrl_c()
			.expect("failed to register CTRL+C handler");
		info!("Registered CTRL+C handler");

		let mut ctrl_close = tokio::signal::windows::ctrl_close()
			.expect("failed to register CTRL+CLOSE handler");
		info!("Registered CTRL+CLOSE handler");

		let mut ctrl_logoff = tokio::signal::windows::ctrl_logoff()
			.expect("failed to register CTRL+LOGOFF handler");
		info!("Registered CTRL+LOGOFF handler");

		let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown()
			.expect("failed to register CTRL+SHUTDOWN handler");
		info!("Registered CTRL+SHUTDOWN handler");

		info!("All Windows signal handlers registered successfully, waiting for signals");

		let mut shutdown_rx = bot_data.shutdown_signal.subscribe();

		tokio::select! {
			_ = ctrl_break.recv() => { info!("Received CTRL+BREAK signal"); },
			_ = ctrl_c.recv() => { info!("Received CTRL+C signal"); },
			_ = ctrl_close.recv() => { info!("Received CTRL+CLOSE signal"); },
			_ = ctrl_logoff.recv() => { info!("Received CTRL+LOGOFF signal"); },
			_ = ctrl_shutdown.recv() => { info!("Received CTRL+SHUTDOWN signal"); },
			_ = shutdown_rx.recv() => { info!("Received internal shutdown signal"); },
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		info!("Sending shutdown signal to all background tasks");
		if let Err(e) = bot_data.shutdown_signal.send(()) {
			warn!("Failed to send shutdown signal: {}", e);
		} else {
			info!("Shutdown signal sent successfully");
		}

		info!("Waiting for background tasks to shut down gracefully");
		tokio::time::sleep(Duration::from_secs(2)).await;
		info!("Proceeding with bot shutdown");
	}

	Ok(())
}

async fn init_db(db_config: DbConfig) -> anyhow::Result<()> {
	let url = db_config.get_url()?;

	#[cfg(windows)]
	let binary_name = "./migration.exe";

	#[cfg(unix)]
	let binary_name = if cfg!(debug_assertions) {
		"./migration"
	} else {
		"migration"
	};

	let child = process::Command::new(binary_name)
		.env("DATABASE_URL", &url)
		.spawn()
		.context("Failed to run Migration")?;

	child
		.wait_with_output()
		.context("Failed to wait for Migration")?;

	Ok(())
}

