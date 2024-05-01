use std::sync::Arc;

use serenity::all::ShardManager;
use serenity::prelude::TypeMapKey;

/// `ShardManagerContainer` is a struct that does not hold any data itself.
/// It is used as a key to access the `ShardManager` in the `TypeMap` of the `Context` data.
pub struct ShardManagerContainer;

/// This implementation allows `ShardManagerContainer` to be used as a key in `TypeMap`.
/// The associated `Value` type is `Arc<ShardManager>`, which means that the value
/// that will be stored in the `TypeMap` under this key is an `Arc<ShardManager>`.
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
