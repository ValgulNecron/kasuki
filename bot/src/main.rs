use crate::config::{Config, DbConfig};
use crate::constant::{CACHE_MAX_CAPACITY, COMMAND_USE_PATH, TIME_BETWEEN_CACHE_UPDATE};
use crate::event_handler::{BotData, Handler, RootUsage};
use crate::logger::{create_log_directory, init_logger};
use anyhow::{Context, Result};
use moka::future::Cache;
use serenity::gateway::ShardManager;
use serenity::prelude::GatewayIntents;
use serenity::secrets::Token;
use serenity::Client;
use songbird::driver::DecodeMode;
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{error, info};

pub mod autocomplete;
mod background_task;
mod command;
mod components;
mod config;
mod constant;
mod custom_serenity_impl;
pub mod database;
pub mod error_management;
mod event_handler;
mod helper;
mod logger;
mod register;
mod structure;

#[tokio::main]

async fn main() {
	println!("Preparing bot environment please wait...");

	// read config.toml as string
	let config = match std::fs::read_to_string("config.toml") {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error while reading config.toml: {:?}", e);

			process::exit(1);
		},
	};

	let mut config: Config = match toml::from_str(&config) {
		Ok(config) => config,
		Err(e) => {
			eprintln!("Error while parsing config.toml: {:?}", e);

			process::exit(1);
		},
	};

	config.set_default_value_on_none();

	let log = config.logging.log_level.clone();

	let discord_token = config.bot.discord_token.clone();

	let max_log_retention_days = config.logging.max_log_retention;

	let config = Arc::new(config);

	// Get the log level from the environment variable "RUST_LOG".
	// If the variable is not set, default to "info".
	let log = log.as_str();

	// Create the log directory.
	// If an error occurs, print the error and return.
	if let Err(e) = create_log_directory() {
		eprintln!("{:?}", e);

		process::exit(2);
	}

	// Initialize the logger with the specified log level.
	// If an error occurs, print the error and return.
	let _guard = match init_logger(log, max_log_retention_days) {
		Ok(guard) => guard,
		Err(e) => {
			eprintln!("{:?}", e);

			process::exit(2);
		},
	};

	// Initialize the SQL database.
	// If an error occurs, log the error and return.
	if let Err(e) = init_db(config.clone()).await {
		let e = e.to_string().replace("\\\\n", "\n");

		error!("{}", e);

		process::exit(4);
	}

	let number_of_command_use_per_command: RootUsage;

	// populate the number_of_command_use_per_command with the content of the file
	if let Ok(content) = std::fs::read_to_string(COMMAND_USE_PATH) {
		number_of_command_use_per_command =
			serde_json::from_str(&content).unwrap_or_else(|_| RootUsage::new());
	} else {
		number_of_command_use_per_command = RootUsage::new();
	}

	let number_of_command_use_per_command =
		Arc::new(RwLock::new(number_of_command_use_per_command));

	let cache: Cache<String, String> = Cache::builder()
		.time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
		.max_capacity(CACHE_MAX_CAPACITY)
		.build();

	let anilist_cache: Arc<RwLock<Cache<String, String>>> = Arc::new(RwLock::new(cache));

	let cache: Cache<String, String> = Cache::builder()
		.time_to_live(Duration::from_secs(TIME_BETWEEN_CACHE_UPDATE))
		.max_capacity(CACHE_MAX_CAPACITY)
		.build();

	let vndb_cache: Arc<RwLock<Cache<String, String>>> = Arc::new(RwLock::new(cache));

	let connection = match sea_orm::Database::connect(get_url(config.db.clone())).await {
		Ok(connection) => connection,
		Err(e) => {
			error!("Failed to connect to the database. {}", e);

			return;
		},
	};

	// Get all the non-privileged intent.
	let gateway_intent_non_privileged =
		GatewayIntents::non_privileged() | GatewayIntents::GUILD_VOICE_STATES;

	// Get the needed privileged intent.
	let gateway_intent_privileged = GatewayIntents::GUILD_MEMBERS
        // | GatewayIntents::GUILD_PRESENCES
        //         | GatewayIntents::MESSAGE_CONTENT
        ;

	// Combine both intents for the client to consume.
	let mut intent = gateway_intent_non_privileged;

	intent |= gateway_intent_privileged;

	let gateway_intent = intent;

	// Log a message indicating the bot is starting.
	info!("Finished preparing the environment. Starting the bot.");

	// Create a new client instance using the provided token and gateway intents.
	// The client is built with an event handler of type `Handler`.
	// If the client creation fails, log the error and exit the process.
	let discord_token = match Token::from_str(discord_token.as_str()) {
		Ok(token) => token,
		Err(e) => {
			error!("Failed to get the token. {}", e);

			return;
		},
	};

	let songbird_config = songbird::Config::default().decode_mode(DecodeMode::Decode);

	let manager = songbird::Songbird::serenity_from_config(songbird_config);

	let bot_data: Arc<BotData> = Arc::new(BotData {
		number_of_command_use_per_command,
		config,
		bot_info: Arc::new(RwLock::new(None)),
		anilist_cache,
		vndb_cache,
		already_launched: false.into(),
		apps: Arc::new(Default::default()),
		user_blacklist_server_image: Arc::new(Default::default()),
		db_connection: Arc::new(connection),
		manager: Arc::clone(&manager),
		http_client: reqwest::Client::new(),
		shard_manager: Arc::new(Default::default()),
	});

	let mut client = Client::builder(discord_token, gateway_intent)
		.data(bot_data.clone())
		.voice_manager::<songbird::Songbird>(Arc::clone(&manager))
		.event_handler(Handler)
		.await
		.unwrap_or_else(|e| {
			error!("Error while creating client: {}", e);

			process::exit(5);
		});

	let shard_manager = client.shard_manager.clone();

	let bot_data_manager = bot_data.shard_manager.clone();
	let mut guard = bot_data_manager.write().await;

	*guard = Some(shard_manager);
	// Clone the shard manager from the client.
	let shard_manager = client.shard_manager.clone();

	let shutdown = shard_manager.clone();

	// Spawn a new asynchronous task for starting the client.
	// If the client fails to start, log the error.
	tokio::spawn(async move {
		if let Err(why) = client.start_autosharded().await {
			error!("Client error: {:?}", why);

			process::exit(6);
		}
	});

	#[cfg(unix)]
	{
		// Create a signal handler for "all" signals in unix.
		// If a signal is received, print a shutdown message.
		// All signals and not only ctrl-c
		let mut sigint =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

		let mut sigterm =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();

		let mut sigquit =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::quit()).unwrap();

		let mut sigusr1 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()).unwrap();

		let mut sigusr2 =
			tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined2()).unwrap();

		tokio::select! {
			_ = sigint.recv() => {},
			_ = sigterm.recv() => {},
			_ = sigquit.recv() => {},
			_ = sigusr1.recv() => {},
			_ = sigusr2.recv() => {},
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		ShardManager::shutdown_all(&shutdown).await;

		std::process::exit(0);
	}

	#[cfg(windows)]
	{
		// Create a signal handler for "all" signals in windows.
		// If a signal is received, print a shutdown message.
		// All signals and not only ctrl-c
		let mut ctrl_break = tokio::signal::windows::ctrl_break().unwrap();

		let mut ctrl_c = tokio::signal::windows::ctrl_c().unwrap();

		let mut ctrl_close = tokio::signal::windows::ctrl_close().unwrap();

		let mut ctrl_logoff = tokio::signal::windows::ctrl_logoff().unwrap();

		let mut ctrl_shutdown = tokio::signal::windows::ctrl_shutdown().unwrap();

		tokio::select! {
			_ = ctrl_break.recv() => {},
			_ = ctrl_c.recv() => {},
			_ = ctrl_close.recv() => {},
			_ = ctrl_logoff.recv() => {},
			_ = ctrl_shutdown.recv() => {},
		}

		info!("Received bot shutdown signal. Shutting down bot.");

		ShardManager::shutdown_all(&shutdown).await;

		process::exit(0);
	}
}

async fn init_db(config: Arc<Config>) -> Result<()> {
	let db_config = config.db.clone();

	let url = get_url(db_config);

	std::env::set_var("DATABASE_URL", url);

	#[cfg(windows)]
	{
		let mut cmd = process::Command::new("./Migration.exe");

		let child = cmd.spawn().context("Failed to run Migration")?;

		child
			.wait_with_output()
			.context("Failed to wait for Migration")?;
	}

	#[cfg(unix)]
	{
		let mut cmd = process::Command::new("./Migration");

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
