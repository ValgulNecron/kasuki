use serenity::all::ShardManager;
use std::sync::Arc;

use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
