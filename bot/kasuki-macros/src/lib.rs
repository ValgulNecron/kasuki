mod args;
mod command;
mod command_group;
mod utils;

use proc_macro::TokenStream;
use syn::parse::Parse;
use syn::parse_macro_input;

use crate::utils::CommandAttrs;

/// Marks a function as a top-level slash command (Level 1).
///
/// # Example
///
/// ```ignore
/// #[command]
/// pub async fn anime(
///     ctx: &SerenityContext,
///     interaction: &CommandInteraction,
///     #[autocomplete] anime_name: String,
/// ) -> Result<()> {
///     // ...
/// }
/// ```
///
/// # Attributes
///
/// - `#[command]` — default settings
/// - `#[command(nsfw = true)]` — mark as NSFW
/// - `#[command(guild_only)]` — register as guild-only command
/// - `#[command(contexts = [Guild])]` — restrict interaction contexts
/// - `#[command(permissions = [Administrator])]` — require permissions
///
/// # Parameter attributes
///
/// - `#[autocomplete]` — enable autocomplete for this option
/// - `#[choices(a, b, c)]` — static choice list
/// - `#[desc = "..."]` — override option description (default: Fluent lookup)
/// - `#[name = "..."]` — override the Discord option name (default: param name)
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
	let item_fn = parse_macro_input!(item as syn::ItemFn);

	// Parse the outer attribute args (e.g., #[command(nsfw = true)])
	let attrs = match parse_command_attrs(attr, &item_fn.attrs) {
		Ok(a) => a,
		Err(e) => return e.to_compile_error().into(),
	};

	match command::expand_command(&attrs, item_fn) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Marks a module as a command group (Level 2).
///
/// The module name becomes the parent command name. Functions inside marked
/// with `#[subcommand]` become subcommands. Nested modules marked with
/// `#[subcommand_group]` become Level 3 subcommand groups.
///
/// # Example
///
/// ```ignore
/// #[command_group]
/// mod bot {
///     #[subcommand]
///     pub async fn credit(ctx: &SerenityContext, interaction: &CommandInteraction) -> Result<()> {
///         // ...
///     }
///
///     #[subcommand]
///     pub async fn ping(ctx: &SerenityContext, interaction: &CommandInteraction) -> Result<()> {
///         // ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn command_group(attr: TokenStream, item: TokenStream) -> TokenStream {
	let item_mod = parse_macro_input!(item as syn::ItemMod);

	let attrs = match parse_group_attrs(attr, &item_mod.attrs) {
		Ok(a) => a,
		Err(e) => return e.to_compile_error().into(),
	};

	match command_group::expand_command_group(&attrs, item_mod) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

/// Marks a function as a subcommand within a `#[command_group]`.
///
/// This attribute is processed by the parent `#[command_group]` macro and
/// should not be used standalone. As a proc macro attribute, it is a no-op
/// when used outside a command group (the parent macro strips it).
#[proc_macro_attribute]
pub fn subcommand(_attr: TokenStream, item: TokenStream) -> TokenStream {
	// When used inside #[command_group], this is handled by the parent macro.
	// When used standalone, just pass through unchanged.
	item
}

/// Marks a module as a subcommand group (Level 3) within a `#[command_group]`.
///
/// This attribute is processed by the parent `#[command_group]` macro and
/// should not be used standalone.
#[proc_macro_attribute]
pub fn subcommand_group(_attr: TokenStream, item: TokenStream) -> TokenStream {
	item
}

fn parse_command_attrs(
	attr: TokenStream,
	fn_attrs: &[syn::Attribute],
) -> syn::Result<CommandAttrs> {
	if attr.is_empty() {
		return CommandAttrs::parse(fn_attrs);
	}

	// Parse the inline attribute args like #[command(nsfw = true, guild_only)]
	let mut result = CommandAttrs::default();
	let parser = syn::meta::parser(|meta| {
		if meta.path.is_ident("nsfw") {
			result.nsfw = true;
			Ok(())
		} else if meta.path.is_ident("guild_only") {
			result.guild_only = true;
			Ok(())
		} else if meta.path.is_ident("contexts") {
			let content;
			syn::bracketed!(content in meta.input);
			let idents =
				content.parse_terminated(syn::Ident::parse, syn::Token![,])?;
			result.contexts = idents.into_iter().collect();
			Ok(())
		} else if meta.path.is_ident("permissions") {
			let content;
			syn::bracketed!(content in meta.input);
			let idents =
				content.parse_terminated(syn::Ident::parse, syn::Token![,])?;
			result.permissions = idents.into_iter().collect();
			Ok(())
		} else {
			Err(meta.error("unknown command attribute"))
		}
	});

	syn::parse::Parser::parse(parser, attr)?;
	Ok(result)
}

fn parse_group_attrs(
	attr: TokenStream,
	mod_attrs: &[syn::Attribute],
) -> syn::Result<CommandAttrs> {
	if attr.is_empty() {
		return CommandAttrs::parse(mod_attrs);
	}

	let mut result = CommandAttrs::default();
	let parser = syn::meta::parser(|meta| {
		if meta.path.is_ident("guild_only") {
			result.guild_only = true;
			Ok(())
		} else if meta.path.is_ident("contexts") {
			let content;
			syn::bracketed!(content in meta.input);
			let idents =
				content.parse_terminated(syn::Ident::parse, syn::Token![,])?;
			result.contexts = idents.into_iter().collect();
			Ok(())
		} else if meta.path.is_ident("permissions") {
			let content;
			syn::bracketed!(content in meta.input);
			let idents =
				content.parse_terminated(syn::Ident::parse, syn::Token![,])?;
			result.permissions = idents.into_iter().collect();
			Ok(())
		} else {
			Err(meta.error("unknown command_group attribute"))
		}
	});

	syn::parse::Parser::parse(parser, attr)?;
	Ok(result)
}
