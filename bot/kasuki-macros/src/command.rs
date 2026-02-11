use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemFn, Result};

use crate::args::{build_extraction_tokens, build_option_tokens};
use crate::utils::{extract_params, strip_command_attrs, CommandAttrs};

/// Implements the `#[command]` attribute macro for top-level slash commands.
///
/// Transforms a function like:
/// ```ignore
/// #[command]
/// pub async fn anime(
///     ctx: &SerenityContext,
///     interaction: &CommandInteraction,
///     #[autocomplete] anime_name: String,
/// ) -> Result<()> { ... }
/// ```
///
/// Into the original function (with command attrs stripped) plus an
/// `inventory::submit!` block that registers a `CommandDescriptor`.
pub fn expand_command(attrs: &CommandAttrs, mut item_fn: ItemFn) -> Result<TokenStream> {
	let fn_name = &item_fn.sig.ident;
	let fn_name_str = fn_name.to_string();
	let guild_only = attrs.guild_only;

	// Extract param info before stripping attrs
	let params = extract_params(&item_fn.sig.inputs)?;

	// Strip command-specific attributes from the function signature
	strip_command_attrs(&mut item_fn.sig.inputs);

	// Generate option builders for each param
	let fluent_prefix = &fn_name_str;
	let option_tokens: Vec<TokenStream> = params
		.iter()
		.map(|p| build_option_tokens(p, fluent_prefix))
		.collect();

	// Generate extraction code for the handler wrapper
	let extraction_tokens: Vec<TokenStream> =
		params.iter().map(build_extraction_tokens).collect();

	let param_idents: Vec<&syn::Ident> = params.iter().map(|p| &p.ident).collect();

	// Build the option-adding chain
	let options_chain = if option_tokens.is_empty() {
		quote! {}
	} else {
		quote! {
			#( let cmd = cmd.add_option(#option_tokens); )*
		}
	};

	// Generate the nsfw setting
	let nsfw = attrs.nsfw;

	let output = quote! {
		// Emit the original function with command attrs stripped from params
		#item_fn

		inventory::submit! {
			shared::command_registry::CommandDescriptor {
				dispatch_name: #fn_name_str,
				kind: shared::command_registry::CommandKind::TopLevel,
				guild_only: #guild_only,
				build_fn: Some(|loader: &dyn shared::localization::Loader| {
					use std::str::FromStr;

					let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
					let desc_key = concat!(#fn_name_str, "-desc");
					let description = loader.lookup(&lang_id, desc_key);

					let cmd = serenity::all::CreateCommand::new(#fn_name_str)
						.description(description)
						.nsfw(#nsfw);

					#options_chain

					cmd
				}),
				handler_fn: |ctx, interaction| {
					Box::pin(async move {
						use serenity::all::ResolvedValue;

						let opts = interaction.data.options();
						#(#extraction_tokens)*

						#fn_name(ctx, interaction, #(#param_idents),*).await
					})
				},
			}
		}
	};

	Ok(output)
}
