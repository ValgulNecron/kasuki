use anyhow::Result;
use sea_orm::DatabaseConnection;
use serenity::all::{ComponentInteraction, Context as SerenityContext};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Trait for handling Discord component interactions.
///
/// Implement this trait on a unit struct and register it via `inventory::submit!` to
/// add new component handlers without editing `components_dispatch.rs`.
///
/// # Example
/// ```rust
/// struct MyHandler;
/// impl ComponentHandler for MyHandler {
///     fn prefix(&self) -> &'static str { "my_prefix_" }
///     fn handle<'a>(&'a self, ctx: &'a SerenityContext, interaction: &'a ComponentInteraction, db: Arc<DatabaseConnection>)
///         -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>
///     {
///         Box::pin(async move { /* ... */ Ok(()) })
///     }
/// }
/// inventory::submit! { &MyHandler as &dyn ComponentHandler }
/// ```
pub trait ComponentHandler: Send + Sync + 'static {
	/// The custom_id prefix this handler matches (e.g. `"user_"` or `"next_activity_"`).
	fn prefix(&self) -> &'static str;

	/// Whether to use prefix matching. Defaults to `true`.
	fn match_prefix(&self) -> bool {
		true
	}

	/// Handle the component interaction asynchronously.
	fn handle<'a>(
		&'a self, ctx: &'a SerenityContext, interaction: &'a ComponentInteraction,
		db: Arc<DatabaseConnection>,
	) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

inventory::collect!(&'static dyn ComponentHandler);
