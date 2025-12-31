use std::sync::Arc;
use tokio::sync::RwLock;
use sea_orm::DatabaseConnection;
use serenity::all::{ShardManager};
use shared::config::Config;
use shared::cache::CacheInterface;

pub struct BotData {
    pub config: Arc<Config>,
    pub db_connection: Arc<DatabaseConnection>,
    pub user_blacklist: Arc<RwLock<Vec<String>>>,
    pub anilist_cache: Arc<RwLock<CacheInterface>>,
    pub shard_manager: Arc<RwLock<Option<Arc<ShardManager>>>>,
}

impl BotData {
    pub fn new(config: Config, db: DatabaseConnection) -> Self {
        Self {
            config: Arc::new(config),
            db_connection: Arc::new(db),
            user_blacklist: Arc::new(RwLock::new(Vec::new())),
            anilist_cache: Arc::new(RwLock::new(CacheInterface::new())),
            shard_manager: Arc::new(RwLock::new(None)),
        }
    }
}
