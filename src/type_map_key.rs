use reqwest::Client as HttpClient;
use serenity::gateway::ShardManager;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
