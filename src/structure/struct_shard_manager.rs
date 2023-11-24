use std::sync::Arc;

use serenity::client::bridge::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;

/// ShardManagerContainer
///
/// A struct representing a container for the shard manager.
///
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}
