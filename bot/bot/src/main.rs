use crate::event_handler::{BotData, Handler};
use crate::logger::{create_log_directory, init_logger};
use anyhow::Context;
use shared::cache::CacheInterface;
use shared::config::{Config, DbConfig};

use serenity::all::GatewayIntents;
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
mod command;
mod components;
mod constant;
mod custom_serenity_impl;
pub mod error_management;
mod event_handler;
mod helper;
pub mod launch_task;
mod logger;
mod music_events;
mod register;
mod server_image;
mod structure;

#[tokio::main]
async fn main() {
	rustls::crypto::aws_lc_rs::default_provider()
		.install_default()
		.expect("Failed to install default CryptoProvider");

	println!("Preparing bot environment please wait...");
	let config: Config = match Config::new() {
		Ok(conf) => conf,
		Err(e) => {
			eprintln!("Error while reading config.toml: {:?}", e);
			process::exit(2);
		},
	};
	println!("Config loaded successfully");

	let log = config.logging.log_level.clone();
	let max_log_retention_days = config.logging.max_log_retention;
	if let Err(e) = create_log_directory() {
		eprintln!("{:?}", e);

		process::exit(2);
	}
	let _guard = match init_logger(log.as_str(), max_log_retention_days) {
		Ok(guard) => {
			info!("Logger initialized successfully with level: {}", log);
			guard
		},
		Err(e) => {
			eprintln!("{:?}", e);
			process::exit(2);
		},
	};
	info!("Log retention days: {}", max_log_retention_days);

	info!("Loading locales");
	if let Err(e) = shared::localization::load_locales() {
		error!("Failed to load locales: {}", e);
		process::exit(8);
	}
	info!("Locales loaded successfully");

	let discord_token = config.bot.discord_token.clone();
	info!("Bot token length: {}", discord_token.len());

	info!("Initializing database");
	let db_config = config.db.clone();
	if let Err(e) = init_db(db_config).await {
		let e = e.to_string().replace("\\\\n", "\n");
		error!("Database initialization failed: {}", e);
		process::exit(4);
	}
	info!("Database initialized successfully");

	let _cache_config = config.cache.clone();
	info!("Initializing caches");
	let anilist_cache: Arc<RwLock<CacheInterface>> = Arc::new(RwLock::new(CacheInterface::new()));
	let vndb_cache: Arc<RwLock<CacheInterface>> = Arc::new(RwLock::new(CacheInterface::new()));
	info!("Caches initialized successfully");

	info!("Connecting to database");
	let connection = match sea_orm::Database::connect(get_url(config.db.clone())).await {
		Ok(connection) => {
			info!("Successfully connected to database");
			connection
		},
		Err(e) => {
			error!("Failed to connect to the database: {}", e);
			return;
		},
	};
	info!("Database connection established successfully");

	info!("Configuring Discord gateway intents");
	let gateway_intent_non_privileged =
		GatewayIntents::non_privileged() | GatewayIntents::GUILD_VOICE_STATES;
	info!(
		"Non-privileged intents configured: {:?}",
		gateway_intent_non_privileged
	);

	let gateway_intent_privileged = GatewayIntents::GUILD_MEMBERS
        // | GatewayIntents::GUILD_PRESENCES
        // | GatewayIntents::MESSAGE_CONTENT
        ;
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
	let discord_token = match Token::from_str(discord_token.as_str()) {
		Ok(token) => {
			info!("Discord token parsed successfully");
			token
		},
		Err(e) => {
			error!("Failed to parse Discord token: {}", e);
			return;
		},
	};

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

	info!("Initializing bot data structure");
	let bot_data: Arc<BotData> = Arc::new(BotData {
		config: Arc::from(config.clone()),
		bot_info: Arc::new(RwLock::new(None)),
		anilist_cache,
		vndb_cache,
		already_launched: false.into(),
		apps: Arc::new(Default::default()),
		user_blacklist: Arc::new(Default::default()),
		db_connection: Arc::new(connection),
		manager: Arc::clone(&manager),
		http_client: Arc::from(reqwest::Client::new()),
		shard_manager: Arc::new(Default::default()),
		lavalink: Arc::new(Default::default()),
		shutdown_signal: Arc::new(shutdown_tx),
		vocal_session: Arc::new(Default::default()),
		user_color_update_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
		server_image_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
	});
	info!("Bot data structure initialized successfully");

	info!("Creating Discord client");
	let mut client = Client::builder(discord_token, gateway_intent)
		.data(bot_data.clone())
		.voice_manager(manager)
		.event_handler(Arc::new(Handler))
		.await
		.unwrap_or_else(|e| {
			error!("Error while creating Discord client: {}", e);
			process::exit(5);
		});
	info!("Discord client created successfully");

	let data = bot_data.clone();
	info!("Setting up shard manager");
	let bot_data = data;
	let mut guard = bot_data.shard_manager.write().await;
	let runner = client.shard_manager.runners.clone();
	*guard = Some(runner);
	drop(guard);
	info!("Shard manager configured successfully");

	info!("Starting Discord client with auto-sharding");
	tokio::spawn(async move {
		if let Err(why) = client.start_autosharded().await {
			error!("Discord client error: {:?}", why);
			process::exit(6);
		}

		info!("Discord client shutdown gracefully");
		drop(client);
	});

	#[cfg(unix)]
	{
		info!("Setting up signal handlers for Unix environment");
		// Create a signal handler for "all" signals in unix.
		// If a signal is received, print a shutdown message.
		// All signals and not only ctrl-c
		let mut sigint =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
		info!("Registered SIGINT handler");

		let mut sigterm =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
		info!("Registered SIGTERM handler");

		let mut sigquit =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()).unwrap();
		info!("Registered SIGQUIT handler");

		let mut sigusr1 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();
		info!("Registered SIGUSR1 handler");

		let mut sigusr2 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined2()).unwrap();
		info!("Registered SIGUSR2 handler");

		info!("All Unix signal handlers registered successfully, waiting for signals");

		tokio::select! {
			_ = sigint.recv() => { info!("Received SIGINT signal"); },
			_ = sigterm.recv() => { info!("Received SIGTERM signal"); },
			_ = sigquit.recv() => { info!("Received SIGQUIT signal"); },
			_ = sigusr1.recv() => { info!("Received SIGUSR1 signal"); },
			_ = sigusr2.recv() => { info!("Received SIGUSR2 signal"); },
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		// Send shutdown signal to all background tasks
		info!("Sending shutdown signal to all background tasks");
		if let Err(e) = bot_data.shutdown_signal.send(()) {
			warn!("Failed to send shutdown signal: {}", e);
		} else {
			info!("Shutdown signal sent successfully");
		}

		// Wait a moment for tasks to clean up
		info!("Waiting for background tasks to shut down gracefully");
		tokio::time::sleep(Duration::from_secs(2)).await;
		info!("Proceeding with bot shutdown");
	}

	#[cfg(windows)]
	{
		info!("Setting up signal handlers for Windows environment");
		// Create a signal handler for "all" signals in windows.
		// If a signal is received, print a shutdown message.
		// All signals and not only ctrl-c
		let mut ctrl_break = tokio::signal::windows::ctrl_break().unwrap();
		info!("Registered CTRL+BREAK handler");

		let mut ctrl_c = tokio::signal::windows::ctrl_c().unwrap();
		info!("Registered CTRL+C handler");

		let mut ctrl_close = tokio::signal::windows::ctrl_close().unwrap();
		info!("Registered CTRL+CLOSE handler");

		let mut ctrl_logoff = tokio::signal::windows::ctrl_logoff().unwrap();
		info!("Registered CTRL+LOGOFF handler");

		let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown().unwrap();
		info!("Registered CTRL+SHUTDOWN handler");

		info!("All Windows signal handlers registered successfully, waiting for signals");

		tokio::select! {
			_ = ctrl_break.recv() => { info!("Received CTRL+BREAK signal"); },
			_ = ctrl_c.recv() => { info!("Received CTRL+C signal"); },
			_ = ctrl_close.recv() => { info!("Received CTRL+CLOSE signal"); },
			_ = ctrl_logoff.recv() => { info!("Received CTRL+LOGOFF signal"); },
			_ = ctrl_shutdown.recv() => { info!("Received CTRL+SHUTDOWN signal"); },
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		// Send shutdown signal to all background tasks
		info!("Sending shutdown signal to all background tasks");
		if let Err(e) = bot_data.shutdown_signal.send(()) {
			warn!("Failed to send shutdown signal: {}", e);
		} else {
			info!("Shutdown signal sent successfully");
		}

		// Wait a moment for tasks to clean up
		info!("Waiting for background tasks to shut down gracefully");
		tokio::time::sleep(Duration::from_secs(2)).await;
		info!("Proceeding with bot shutdown");
	}
}

async fn init_db(db_config: DbConfig) -> anyhow::Result<()> {
	let url = get_url(db_config);
	unsafe {
		std::env::set_var("DATABASE_URL", url);
	}
	// check if the env var is set
	match std::env::var("DATABASE_URL") {
		Ok(_) => {},
		Err(e) => {
			println!("DATABASE_URL is not set: {}", e);
			std::process::exit(1);
		},
	}

	#[cfg(windows)]
	{
		let mut cmd = process::Command::new("./migration.exe");

		let child = cmd.spawn().context("Failed to run Migration")?;

		child
			.wait_with_output()
			.context("Failed to wait for Migration")?;
	}

	#[cfg(unix)]
	{
		let binary_name = if cfg!(debug_assertions) {
			"./migration"
		} else {
			"migration"
		};
		let mut cmd = process::Command::new(binary_name);

		let child = cmd.spawn().context("Failed to run Migration")?;

		child
			.wait_with_output()
			.context("Failed to wait for Migration")?;
	}

	Ok(())
}

pub fn get_url(db_config: DbConfig) -> String {
	match db_config.db_type.as_str() {
		"postgresql" => {
			let host = match db_config.host.clone() {
				Some(host) => host,
				None => {
					error!("No host provided");

					process::exit(7)
				},
			};

			let port = match db_config.port {
				Some(port) => port,
				None => {
					error!("No port provided");

					process::exit(7)
				},
			};

			let user = match db_config.user.clone() {
				Some(user) => user,
				None => {
					error!("No user provided");

					process::exit(7)
				},
			};

			let password = match db_config.password.clone() {
				Some(password) => password,
				None => {
					error!("No password provided");

					process::exit(7)
				},
			};

			let db_name = db_config.database.unwrap_or(String::from("kasuki"));

			let param = vec![("user", user.as_str()), ("password", password.as_str())];

			let param = serde_urlencoded::to_string(&param).unwrap();

			let url = format!("postgresql://{}:{}/{}?{}", host, port, db_name, param);

			url
		},
		_ => {
			panic!("Unsupported database type");
		},
	}
}
