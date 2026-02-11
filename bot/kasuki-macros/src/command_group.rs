use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Item, ItemMod, Result};

use crate::args::{build_extraction_tokens, build_option_tokens};
use crate::utils::{extract_params, strip_command_attrs, CommandAttrs};

/// Implements `#[command_group]` for Level 2 command groups.
///
/// Transforms a module like:
/// ```ignore
/// #[command_group]
/// mod bot {
///     #[subcommand]
///     pub async fn credit(...) -> Result<()> { ... }
///     #[subcommand]
///     pub async fn ping(...) -> Result<()> { ... }
/// }
/// ```
///
/// Into the module (with subcommand functions) plus `inventory::submit!` blocks
/// for the parent command and each subcommand.
pub fn expand_command_group(attrs: &CommandAttrs, item_mod: ItemMod) -> Result<TokenStream> {
	let mod_name = &item_mod.ident;
	let mod_name_str = mod_name.to_string();
	let guild_only = attrs.guild_only;
	let vis = &item_mod.vis;

	let Some((_, ref items)) = item_mod.content else {
		return Err(syn::Error::new_spanned(
			&item_mod,
			"#[command_group] requires an inline module (mod name { ... })",
		));
	};

	let mut subcommand_fns = Vec::new();
	let mut subcommand_group_mods = Vec::new();
	let mut other_items = Vec::new();

	for item in items {
		match item {
			Item::Fn(f) if has_attr(&f.attrs, "subcommand") => {
				subcommand_fns.push(f.clone());
			}
			Item::Mod(m) if has_attr(&m.attrs, "subcommand_group") => {
				subcommand_group_mods.push(m.clone());
			}
			other => {
				other_items.push(other.clone());
			}
		}
	}

	// Process subcommands (Level 2 leaf functions)
	let mut subcmd_option_builders = Vec::new();
	let mut subcmd_descriptors = Vec::new();
	let mut cleaned_fns = Vec::new();

	for mut func in subcommand_fns {
		let fn_name = &func.sig.ident;
		let fn_name_str = fn_name.to_string();
		let dispatch_name = format!("{}_{}", mod_name_str, fn_name_str);
		let fluent_prefix = &dispatch_name;

		// Remove the #[subcommand] attribute from the function
		func.attrs.retain(|a| !a.path().is_ident("subcommand"));

		let params = extract_params(&func.sig.inputs)?;
		strip_command_attrs(&mut func.sig.inputs);

		// Build option tokens for this subcommand's parameters
		let param_option_tokens: Vec<TokenStream> = params
			.iter()
			.map(|p| build_option_tokens(p, fluent_prefix))
			.collect();

		let extraction_tokens: Vec<TokenStream> =
			params.iter().map(build_extraction_tokens).collect();

		let param_idents: Vec<&syn::Ident> = params.iter().map(|p| &p.ident).collect();

		let param_options_chain = if param_option_tokens.is_empty() {
			quote! {}
		} else {
			quote! {
				#( let sub_opt = sub_opt.add_sub_option(#param_option_tokens); )*
			}
		};

		// Generate the SubCommand option for the parent's build_fn
		let desc_key = format!("{}-desc", fn_name_str);
		subcmd_option_builders.push(quote! {
			{
				let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
				let sub_opt = serenity::all::CreateCommandOption::new(
					serenity::all::CommandOptionType::SubCommand,
					#fn_name_str,
					loader.lookup(&lang_id, #desc_key),
				);
				#param_options_chain
				sub_opt
			}
		});

		// Generate a handler descriptor for this subcommand
		let mod_ident = format_ident!("{}", mod_name_str);
		subcmd_descriptors.push(quote! {
			inventory::submit! {
				shared::command_registry::CommandDescriptor {
					dispatch_name: #dispatch_name,
					kind: shared::command_registry::CommandKind::Subcommand,
					guild_only: #guild_only,
					build_fn: None,
					handler_fn: |ctx, interaction| {
						Box::pin(async move {
							use serenity::all::ResolvedValue;

							// For subcommands, options are nested under the subcommand
							let top_opts = interaction.data.options();
							let sub_opt = top_opts.iter()
								.find(|o| o.name == #fn_name_str)
								.ok_or_else(|| anyhow::anyhow!("Missing subcommand: {}", #fn_name_str))?;
							let opts = match &sub_opt.value {
								ResolvedValue::SubCommand(opts) => opts.as_slice(),
								_ => &[],
							};

							#(#extraction_tokens)*

							#mod_ident :: #fn_name(ctx, interaction, #(#param_idents),*).await
						})
					},
				}
			}
		});

		cleaned_fns.push(func);
	}

	// Process subcommand groups (Level 3)
	let mut subcmd_group_option_builders = Vec::new();
	let mut subcmd_group_descriptors = Vec::new();
	let mut subcmd_group_modules = Vec::new();

	for subgroup_mod in subcommand_group_mods {
		let (group_option, group_descriptors, cleaned_mod) =
			expand_subcommand_group(&mod_name_str, guild_only, subgroup_mod)?;
		subcmd_group_option_builders.push(group_option);
		subcmd_group_descriptors.extend(group_descriptors);
		subcmd_group_modules.push(cleaned_mod);
	}

	let output = quote! {
		#vis mod #mod_name {
			use super::*;

			#(#other_items)*
			#(#cleaned_fns)*
			#(#subcmd_group_modules)*
		}

		// Top-level command descriptor
		inventory::submit! {
			shared::command_registry::CommandDescriptor {
				dispatch_name: #mod_name_str,
				kind: shared::command_registry::CommandKind::TopLevel,
				guild_only: #guild_only,
				build_fn: Some(|loader: &dyn shared::localization::Loader| {
					use std::str::FromStr;

					let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
					let desc_key = concat!(#mod_name_str, "-desc");
					let description = loader.lookup(&lang_id, desc_key);

					let cmd = serenity::all::CreateCommand::new(#mod_name_str)
						.description(description);

					// Add subcommand options
					#( let cmd = cmd.add_option(#subcmd_option_builders); )*

					// Add subcommand group options
					#( let cmd = cmd.add_option(#subcmd_group_option_builders); )*

					cmd
				}),
				handler_fn: |_ctx, _interaction| {
					Box::pin(async move {
						// Top-level group commands are not directly invocable;
						// dispatch goes to individual subcommands.
						Err(anyhow::anyhow!("Group command cannot be invoked directly"))
					})
				},
			}
		}

		// Subcommand descriptors
		#(#subcmd_descriptors)*

		// Subcommand group descriptors
		#(#subcmd_group_descriptors)*
	};

	Ok(output)
}

/// Expands a `#[subcommand_group]` module nested inside a `#[command_group]`.
///
/// Returns:
/// - The `CreateCommandOption` token for embedding in the parent's `build_fn`
/// - A list of `inventory::submit!` descriptors for each leaf subcommand
/// - The cleaned module item
fn expand_subcommand_group(
	parent_name: &str,
	guild_only: bool,
	mut item_mod: ItemMod,
) -> Result<(TokenStream, Vec<TokenStream>, TokenStream)> {
	let group_name = &item_mod.ident;
	let group_name_str = group_name.to_string();
	let vis = &item_mod.vis;

	// Remove #[subcommand_group] attr
	item_mod
		.attrs
		.retain(|a| !a.path().is_ident("subcommand_group"));

	let Some((_, ref items)) = item_mod.content else {
		return Err(syn::Error::new_spanned(
			&item_mod,
			"#[subcommand_group] requires an inline module",
		));
	};

	let mut subcmd_fns = Vec::new();
	let mut other_items = Vec::new();

	for item in items {
		match item {
			Item::Fn(f) if has_attr(&f.attrs, "subcommand") => {
				subcmd_fns.push(f.clone());
			}
			other => {
				other_items.push(other.clone());
			}
		}
	}

	let mut sub_option_builders = Vec::new();
	let mut descriptors = Vec::new();
	let mut cleaned_fns = Vec::new();

	for mut func in subcmd_fns {
		let fn_name = &func.sig.ident;
		let fn_name_str = fn_name.to_string();
		let dispatch_name = format!("{}_{}_{}", parent_name, group_name_str, fn_name_str);
		let fluent_prefix = &dispatch_name;

		func.attrs.retain(|a| !a.path().is_ident("subcommand"));

		let params = extract_params(&func.sig.inputs)?;
		strip_command_attrs(&mut func.sig.inputs);

		let param_option_tokens: Vec<TokenStream> = params
			.iter()
			.map(|p| build_option_tokens(p, fluent_prefix))
			.collect();

		let extraction_tokens: Vec<TokenStream> =
			params.iter().map(build_extraction_tokens).collect();

		let param_idents: Vec<&syn::Ident> = params.iter().map(|p| &p.ident).collect();

		let param_options_chain = if param_option_tokens.is_empty() {
			quote! {}
		} else {
			quote! {
				#( let sub_opt = sub_opt.add_sub_option(#param_option_tokens); )*
			}
		};

		let desc_key = format!("{}-desc", fn_name_str);
		sub_option_builders.push(quote! {
			{
				let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
				let sub_opt = serenity::all::CreateCommandOption::new(
					serenity::all::CommandOptionType::SubCommand,
					#fn_name_str,
					loader.lookup(&lang_id, #desc_key),
				);
				#param_options_chain
				sub_opt
			}
		});

		let parent_ident = format_ident!("{}", parent_name);
		let group_ident = format_ident!("{}", group_name_str);
		descriptors.push(quote! {
			inventory::submit! {
				shared::command_registry::CommandDescriptor {
					dispatch_name: #dispatch_name,
					kind: shared::command_registry::CommandKind::Subcommand,
					guild_only: #guild_only,
					build_fn: None,
					handler_fn: |ctx, interaction| {
						Box::pin(async move {
							use serenity::all::ResolvedValue;

							// Navigate: top options -> subcommand group -> subcommand
							let top_opts = interaction.data.options();
							let group_opt = top_opts.iter()
								.find(|o| o.name == #group_name_str)
								.ok_or_else(|| anyhow::anyhow!("Missing subcommand group: {}", #group_name_str))?;
							let group_opts = match &group_opt.value {
								ResolvedValue::SubCommandGroup(opts) => opts.as_slice(),
								_ => return Err(anyhow::anyhow!("Expected subcommand group")),
							};
							let sub_opt = group_opts.iter()
								.find(|o| o.name == #fn_name_str)
								.ok_or_else(|| anyhow::anyhow!("Missing subcommand: {}", #fn_name_str))?;
							let opts = match &sub_opt.value {
								ResolvedValue::SubCommand(opts) => opts.as_slice(),
								_ => &[],
							};

							#(#extraction_tokens)*

							#parent_ident :: #group_ident :: #fn_name(ctx, interaction, #(#param_idents),*).await
						})
					},
				}
			}
		});

		cleaned_fns.push(func);
	}

	// Build the SubCommandGroup option for the parent
	let group_desc_key = format!("{}-desc", group_name_str);
	let group_option = quote! {
		{
			let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
			let group_opt = serenity::all::CreateCommandOption::new(
				serenity::all::CommandOptionType::SubCommandGroup,
				#group_name_str,
				loader.lookup(&lang_id, #group_desc_key),
			);
			#( let group_opt = group_opt.add_sub_option(#sub_option_builders); )*
			group_opt
		}
	};

	let cleaned_mod = quote! {
		#vis mod #group_name {
			use super::*;

			#(#other_items)*
			#(#cleaned_fns)*
		}
	};

	Ok((group_option, descriptors, cleaned_mod))
}

/// Checks if an attribute list contains an attribute with the given name.
fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
	attrs.iter().any(|a| a.path().is_ident(name))
}
