use std::sync::Arc;

use serenity::all::ShardManager;
use serenity::prelude::TypeMapKey;

// Define a public struct `ShardManagerContainer`
pub struct ShardManagerContainer;

// Implement the `TypeMapKey` trait for `ShardManagerContainer`
// This allows `ShardManagerContainer` to be used as a key in Serenity's TypeMap
impl TypeMapKey for ShardManagerContainer {
    // Define the associated type `Value` for the `TypeMapKey` trait
    // In this case, the value is an `Arc<ShardManager>`
    // `Arc` is a thread-safe reference-counting pointer
    // `ShardManager` is a struct provided by Serenity that allows for managing shard connections
    type Value = Arc<ShardManager>;
}
