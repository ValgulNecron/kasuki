use std::future::Future;
use std::pin::Pin;

use serenity::all::{CommandInteraction, CreateCommand};
use serenity::prelude::Context as SerenityContext;

use crate::localization::Loader;

/// The type of a command in Discord's hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
	/// A top-level slash command (Level 1), e.g. `/anime`
	TopLevel,
	/// A subcommand within a group (Level 2/3 leaf), e.g. `/bot credit`
	Subcommand,
	/// A subcommand group (Level 2 container), e.g. `/admin anilist`
	SubcommandGroup,
	/// A user context menu command
	UserCommand,
	/// A message context menu command
	MessageCommand,
}

/// A boxed future returned by command handlers.
pub type CommandResult<'a> = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>>;

/// Describes a registered command for auto-discovery via `inventory`.
///
/// Top-level commands have a `build_fn` that constructs the `CreateCommand` for
/// Discord registration (including subcommands/groups as options). Leaf
/// subcommands have a `handler_fn` that dispatches to the user's async function.
///
/// The `dispatch_name` is the flattened key used for handler lookup, e.g.:
/// - `"anime"` for `/anime`
/// - `"bot_credit"` for `/bot credit`
/// - `"admin_anilist_add_anime_activity"` for `/admin anilist add_anime_activity`
pub struct CommandDescriptor {
	/// Flattened dispatch key (e.g. `"admin_anilist_add_anime_activity"`)
	pub dispatch_name: &'static str,

	/// What level of the Discord hierarchy this descriptor represents
	pub kind: CommandKind,

	/// Whether this command should only be registered in guilds
	pub guild_only: bool,

	/// Builds a `CreateCommand` for Discord registration.
	/// Only meaningful for `TopLevel` descriptors — subcommands are embedded
	/// as options inside their parent's `build_fn`.
	/// Takes a `&dyn Loader` for Fluent localization lookups.
	pub build_fn: Option<fn(&dyn Loader) -> CreateCommand>,

	/// Dispatches this command. The wrapper extracts typed args from the
	/// interaction and calls the user's async handler function.
	pub handler_fn: for<'a> fn(&'a SerenityContext, &'a CommandInteraction) -> CommandResult<'a>,
}

// SAFETY: CommandDescriptor contains only function pointers and static references.
unsafe impl Send for CommandDescriptor {}
unsafe impl Sync for CommandDescriptor {}

inventory::collect!(CommandDescriptor);
