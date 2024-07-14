use std::sync::Arc;

use serenity::all::ShardManager;
use serenity::prelude::TypeMapKey;

pub struct ShardManagerContainer;

/// Implementation of `TypeMapKey` for `ShardManagerContainer`.
///
/// This allows `ShardManagerContainer` to be used as a key in Serenity's `TypeMap`.
/// The `TypeMap` is often used for storing data that needs to be accessed across
/// different parts of a Discord bot application.
///
/// # Type Parameters
///
/// - `Value`: The type of value that will be stored in the `TypeMap` under this key.
///   In this case, it is an `Arc<ShardManager>`, which allows for thread-safe shared
///   ownership of the `ShardManager`. The `ShardManager` is responsible for managing
///   the state and lifecycle of shards in a Discord bot.
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
