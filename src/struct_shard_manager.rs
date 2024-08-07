use std::sync::Arc;

use serenity::all::ShardManager;
use serenity::prelude::TypeMapKey;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
