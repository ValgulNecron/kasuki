use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
	braced, bracketed, parenthesized, parse_macro_input, Expr, Ident, LitBool, LitInt, LitStr,
	Token, Type,
};

// ─── Attribute parsing structures ────────────────────────────────────────────

struct SlashCommandAttr {
	name: String,
	desc: Option<String>,
	struct_name: Option<Ident>,
	command_type: CommandTypeDef,
	nsfw: bool,
	permissions: Vec<Ident>,
	contexts: Vec<Ident>,
	install_contexts: Vec<Ident>,
	args: Vec<ArgDef>,
	extra_fields: Vec<ExtraFieldDef>,
}

#[derive(Clone)]
enum CommandTypeDef {
	ChatInput,
	SubCommand { parent: String },
	SubCommandGroup { parent: String, group: String },
	User,
	Message,
	GuildChatInput { guild_id: u64 },
}

struct ArgDef {
	name: String,
	desc: String,
	arg_type: Ident,
	required: bool,
	autocomplete: bool,
	choices: Vec<ChoiceDef>,
}

struct ChoiceDef {
	name: String,
}

struct ExtraFieldDef {
	field_name: Ident,
	field_type: Type,
	init_expr: Expr,
}

// ─── Parsing helpers ─────────────────────────────────────────────────────────

impl Parse for SlashCommandAttr {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut name = None;
		let mut desc = None;
		let mut struct_name = None;
		let mut command_type = CommandTypeDef::ChatInput;
		let mut nsfw = false;
		let mut permissions = Vec::new();
		let mut contexts = Vec::new();
		let mut install_contexts = Vec::new();
		let mut args = Vec::new();
		let mut extra_fields = Vec::new();

		while !input.is_empty() {
			let key: Ident = input.parse()?;
			input.parse::<Token![=]>()?;

			match key.to_string().as_str() {
				"name" => {
					let val: LitStr = input.parse()?;
					name = Some(val.value());
				},
				"desc" => {
					let val: LitStr = input.parse()?;
					desc = Some(val.value());
				},
				"struct_name" => {
					let val: Ident = input.parse()?;
					struct_name = Some(val);
				},
				"command_type" => {
					command_type = parse_command_type(input)?;
				},
				"nsfw" => {
					let val: LitBool = input.parse()?;
					nsfw = val.value;
				},
				"permissions" => {
					let content;
					bracketed!(content in input);
					let items: Punctuated<Ident, Token![,]> =
						content.parse_terminated(Ident::parse, Token![,])?;
					permissions = items.into_iter().collect();
				},
				"contexts" => {
					let content;
					bracketed!(content in input);
					let items: Punctuated<Ident, Token![,]> =
						content.parse_terminated(Ident::parse, Token![,])?;
					contexts = items.into_iter().collect();
				},
				"install_contexts" => {
					let content;
					bracketed!(content in input);
					let items: Punctuated<Ident, Token![,]> =
						content.parse_terminated(Ident::parse, Token![,])?;
					install_contexts = items.into_iter().collect();
				},
				"args" => {
					let content;
					bracketed!(content in input);
					while !content.is_empty() {
						args.push(parse_arg(&content)?);
						if !content.is_empty() {
							content.parse::<Token![,]>()?;
						}
					}
				},
				"extra_fields" => {
					let content;
					bracketed!(content in input);
					while !content.is_empty() {
						extra_fields.push(parse_extra_field(&content)?);
						if !content.is_empty() {
							content.parse::<Token![,]>()?;
						}
					}
				},
				other => {
					return Err(syn::Error::new(
						key.span(),
						format!("unknown attribute key: {}", other),
					));
				},
			}

			if !input.is_empty() {
				input.parse::<Token![,]>()?;
			}
		}

		Ok(SlashCommandAttr {
			name: name.ok_or_else(|| syn::Error::new(input.span(), "missing `name`"))?,
			desc,
			struct_name,
			command_type,
			nsfw,
			permissions,
			contexts,
			install_contexts,
			args,
			extra_fields,
		})
	}
}

fn parse_command_type(input: ParseStream) -> syn::Result<CommandTypeDef> {
	let type_name: Ident = input.parse()?;
	match type_name.to_string().as_str() {
		"ChatInput" => Ok(CommandTypeDef::ChatInput),
		"User" => Ok(CommandTypeDef::User),
		"Message" => Ok(CommandTypeDef::Message),
		"SubCommand" => {
			let content;
			parenthesized!(content in input);
			let mut parent = String::new();
			while !content.is_empty() {
				let k: Ident = content.parse()?;
				content.parse::<Token![=]>()?;
				let v: LitStr = content.parse()?;
				match k.to_string().as_str() {
					"parent" => parent = v.value(),
					_ => {},
				}
				if !content.is_empty() {
					content.parse::<Token![,]>()?;
				}
			}
			Ok(CommandTypeDef::SubCommand { parent })
		},
		"SubCommandGroup" => {
			let content;
			parenthesized!(content in input);
			let mut parent = String::new();
			let mut group = String::new();
			while !content.is_empty() {
				let k: Ident = content.parse()?;
				content.parse::<Token![=]>()?;
				let v: LitStr = content.parse()?;
				match k.to_string().as_str() {
					"parent" => parent = v.value(),
					"group" => group = v.value(),
					_ => {},
				}
				if !content.is_empty() {
					content.parse::<Token![,]>()?;
				}
			}
			Ok(CommandTypeDef::SubCommandGroup { parent, group })
		},
		"GuildChatInput" => {
			let content;
			braced!(content in input);
			let mut guild_id = 0u64;
			while !content.is_empty() {
				let k: Ident = content.parse()?;
				content.parse::<Token![=]>()?;
				let v: LitInt = content.parse()?;
				match k.to_string().as_str() {
					"guild_id" => guild_id = v.base10_parse()?,
					_ => {},
				}
				if !content.is_empty() {
					content.parse::<Token![,]>()?;
				}
			}
			Ok(CommandTypeDef::GuildChatInput { guild_id })
		},
		_ => Err(syn::Error::new(
			type_name.span(),
			format!("unknown command_type: {}", type_name),
		)),
	}
}

fn parse_arg(input: ParseStream) -> syn::Result<ArgDef> {
	let content;
	parenthesized!(content in input);

	let mut name = String::new();
	let mut desc = String::new();
	let mut arg_type = format_ident!("String");
	let mut required = true;
	let mut autocomplete = false;
	let mut choices = Vec::new();

	while !content.is_empty() {
		let key: Ident = content.parse()?;
		content.parse::<Token![=]>()?;

		match key.to_string().as_str() {
			"name" => {
				let v: LitStr = content.parse()?;
				name = v.value();
			},
			"desc" => {
				let v: LitStr = content.parse()?;
				desc = v.value();
			},
			"arg_type" => {
				arg_type = content.parse()?;
			},
			"required" => {
				let v: LitBool = content.parse()?;
				required = v.value;
			},
			"autocomplete" => {
				let v: LitBool = content.parse()?;
				autocomplete = v.value;
			},
			"choices" => {
				let inner;
				bracketed!(inner in content);
				while !inner.is_empty() {
					choices.push(parse_choice(&inner)?);
					if !inner.is_empty() {
						inner.parse::<Token![,]>()?;
					}
				}
			},
			_ => {
				return Err(syn::Error::new(
					key.span(),
					format!("unknown arg key: {}", key),
				));
			},
		}

		if !content.is_empty() {
			content.parse::<Token![,]>()?;
		}
	}

	Ok(ArgDef {
		name,
		desc,
		arg_type,
		required,
		autocomplete,
		choices,
	})
}

fn parse_choice(input: ParseStream) -> syn::Result<ChoiceDef> {
	let content;
	parenthesized!(content in input);
	let mut name = String::new();

	while !content.is_empty() {
		let key: Ident = content.parse()?;
		content.parse::<Token![=]>()?;

		match key.to_string().as_str() {
			"name" => {
				let v: LitStr = content.parse()?;
				name = v.value();
			},
			_ => {
				return Err(syn::Error::new(
					key.span(),
					format!("unknown choice key: {}", key),
				));
			},
		}

		if !content.is_empty() {
			content.parse::<Token![,]>()?;
		}
	}

	Ok(ChoiceDef { name })
}

fn parse_extra_field(input: ParseStream) -> syn::Result<ExtraFieldDef> {
	let field_name: Ident = input.parse()?;
	input.parse::<Token![:]>()?;
	let field_type: Type = input.parse()?;
	input.parse::<Token![=]>()?;
	let init_expr: Expr = input.parse()?;
	Ok(ExtraFieldDef {
		field_name,
		field_type,
		init_expr,
	})
}

// ─── The main proc macro ────────────────────────────────────────────────────

/// Attribute macro that generates a command struct, `impl Command`, a `SlashCommand` registry
/// entry, and `inventory::submit!` for auto-registration and dispatch.
///
/// # Example
/// ```ignore
/// #[slash_command(
///     name = "ping", desc = "Pong!",
///     command_type = SubCommand(parent = "bot"),
///     contexts = [Guild, BotDm, PrivateChannel],
///     install_contexts = [Guild, User],
/// )]
/// async fn ping_command(self_: PingCommand) -> Result<EmbedsContents<'_>> {
///     // ... body ...
/// }
/// ```
#[proc_macro_attribute]
pub fn slash_command(attr: TokenStream, item: TokenStream) -> TokenStream {
	let attrs = parse_macro_input!(attr as SlashCommandAttr);
	let input_fn = parse_macro_input!(item as syn::ItemFn);

	match generate(attrs, input_fn) {
		Ok(tokens) => tokens.into(),
		Err(e) => e.to_compile_error().into(),
	}
}

fn generate(
	attrs: SlashCommandAttr, input_fn: syn::ItemFn,
) -> syn::Result<proc_macro2::TokenStream> {
	let fn_name = &input_fn.sig.ident;
	let fn_body = &input_fn.block;

	// Determine the struct name from the first parameter type annotation
	let struct_name = if let Some(sn) = &attrs.struct_name {
		sn.clone()
	} else {
		// Extract from first param: `self_: FooCommand`
		let first_param = input_fn.sig.inputs.first().ok_or_else(|| {
			syn::Error::new_spanned(&input_fn.sig, "expected at least one parameter")
		})?;

		match first_param {
			syn::FnArg::Typed(pat_type) => {
				if let syn::Type::Path(tp) = pat_type.ty.as_ref() {
					tp.path
						.segments
						.last()
						.map(|s| s.ident.clone())
						.ok_or_else(|| {
							syn::Error::new_spanned(pat_type, "cannot determine struct name")
						})?
				} else {
					return Err(syn::Error::new_spanned(pat_type, "expected a type path"));
				}
			},
			_ => {
				return Err(syn::Error::new_spanned(
					first_param,
					"expected typed parameter",
				))
			},
		}
	};

	// Extract the self_ parameter name
	let self_param_name = {
		let first_param = input_fn.sig.inputs.first().unwrap();
		match first_param {
			syn::FnArg::Typed(pat_type) => {
				if let syn::Pat::Ident(pi) = pat_type.pat.as_ref() {
					pi.ident.clone()
				} else {
					format_ident!("self_")
				}
			},
			_ => format_ident!("self_"),
		}
	};

	// Build extra field definitions and struct field tokens
	let extra_field_defs: Vec<_> = attrs
		.extra_fields
		.iter()
		.map(|ef| {
			let name = &ef.field_name;
			let ty = &ef.field_type;
			quote! { pub #name: #ty }
		})
		.collect();

	let extra_field_inits: Vec<_> = attrs
		.extra_fields
		.iter()
		.map(|ef| {
			let name = &ef.field_name;
			let expr = &ef.init_expr;
			quote! { #name: #expr }
		})
		.collect();

	// Generate struct definition (only if struct_name not reusing existing)
	let struct_def = if attrs.struct_name.is_some() {
		// Reusing an existing struct, don't re-define
		quote! {}
	} else {
		quote! {
			#[derive(Clone)]
			pub struct #struct_name {
				pub ctx: SerenityContext,
				pub command_interaction: CommandInteraction,
				#(#extra_field_defs,)*
			}
		}
	};

	// Generate impl Command — only if we're NOT reusing an existing struct
	// (when reusing, the original definition already has impl Command)
	let impl_command_block = if attrs.struct_name.is_some() {
		quote! {}
	} else {
		let get_contents_closure = quote! { |#self_param_name: #struct_name| async move #fn_body };
		quote! {
			impl crate::command::command::Command for #struct_name {
				fn get_ctx(&self) -> &SerenityContext {
					&self.ctx
				}
				fn get_command_interaction(&self) -> &CommandInteraction {
					&self.command_interaction
				}
				async fn get_contents<'a>(&'a self) -> anyhow::Result<crate::command::embed_content::EmbedsContents<'a>> {
					(#get_contents_closure)(self.clone()).await
				}
			}
		}
	};

	// Build the dispatch key string
	let cmd_name = &attrs.name;
	let dispatch_key = match &attrs.command_type {
		CommandTypeDef::ChatInput => cmd_name.clone(),
		CommandTypeDef::SubCommand { parent } => format!("{}_{}", parent, cmd_name),
		CommandTypeDef::SubCommandGroup { parent, group } => {
			format!("{}_{}_{}", parent, group, cmd_name)
		},
		CommandTypeDef::User => cmd_name.clone(),
		CommandTypeDef::Message => cmd_name.clone(),
		CommandTypeDef::GuildChatInput { .. } => cmd_name.clone(),
	};

	// Determine run method (use fully-qualified path so CommandRun doesn't need to be imported)
	let run_method = match &attrs.command_type {
		CommandTypeDef::User => format_ident!("run_user"),
		_ => format_ident!("run_slash"),
	};

	// Generate the static CommandMeta
	let meta_ident = format_ident!("{}_META", fn_name.to_string().to_uppercase());
	let entry_struct_ident = {
		let pascal = fn_name
			.to_string()
			.split('_')
			.map(|s| {
				let mut c = s.chars();
				match c.next() {
					None => String::new(),
					Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
				}
			})
			.collect::<String>();
		format_ident!("{}Entry", pascal)
	};
	let entry_static_ident = format_ident!("{}_ENTRY", fn_name.to_string().to_uppercase());

	let desc_str = attrs.desc.as_deref().unwrap_or("");
	let nsfw_val = attrs.nsfw;

	let command_type_tokens = match &attrs.command_type {
		CommandTypeDef::ChatInput => {
			quote! { crate::command::registry::DiscordCommandType::ChatInput }
		},
		CommandTypeDef::SubCommand { parent } => {
			quote! { crate::command::registry::DiscordCommandType::SubCommand { parent: #parent } }
		},
		CommandTypeDef::SubCommandGroup { parent, group } => {
			quote! { crate::command::registry::DiscordCommandType::SubCommandGroup { parent: #parent, group: #group } }
		},
		CommandTypeDef::User => quote! { crate::command::registry::DiscordCommandType::User },
		CommandTypeDef::Message => quote! { crate::command::registry::DiscordCommandType::Message },
		CommandTypeDef::GuildChatInput { guild_id } => {
			quote! { crate::command::registry::DiscordCommandType::GuildChatInput { guild_id: #guild_id } }
		},
	};

	// Permissions
	let perm_tokens: Vec<_> = attrs
		.permissions
		.iter()
		.map(|p| {
			quote! { crate::command::registry::PermissionType::#p }
		})
		.collect();

	// Contexts
	let ctx_tokens: Vec<_> = attrs
		.contexts
		.iter()
		.map(|c| {
			quote! { crate::command::registry::ContextType::#c }
		})
		.collect();

	// Install contexts
	let install_ctx_tokens: Vec<_> = attrs
		.install_contexts
		.iter()
		.map(|c| {
			quote! { crate::command::registry::InstallType::#c }
		})
		.collect();

	// Args
	let arg_tokens: Vec<_> = attrs
		.args
		.iter()
		.map(|arg| {
			let aname = &arg.name;
			let adesc = &arg.desc;
			let atype = &arg.arg_type;
			let areq = arg.required;
			let aac = arg.autocomplete;

			let choice_tokens: Vec<_> = arg
				.choices
				.iter()
				.map(|ch| {
					let chname = &ch.name;
					quote! {
						crate::command::registry::ChoiceDef {
							name: #chname,
						}
					}
				})
				.collect();

			quote! {
				crate::command::registry::ArgDef {
					name: #aname,
					desc: #adesc,
					arg_type: crate::command::registry::ArgType::#atype,
					required: #areq,
					autocomplete: #aac,
					choices: &[#(#choice_tokens),*],
				}
			}
		})
		.collect();

	let dispatch_key_str = &dispatch_key;

	let output = quote! {
		#struct_def

		#impl_command_block

		static #meta_ident: crate::command::registry::CommandMeta = crate::command::registry::CommandMeta {
			name: #cmd_name,
			desc: #desc_str,
			command_type: #command_type_tokens,
			nsfw: #nsfw_val,
			permissions: &[#(#perm_tokens),*],
			contexts: &[#(#ctx_tokens),*],
			install_contexts: &[#(#install_ctx_tokens),*],
			args: &[#(#arg_tokens),*],
		};

		struct #entry_struct_ident;

		impl crate::command::registry::SlashCommand for #entry_struct_ident {
			fn meta(&self) -> &'static crate::command::registry::CommandMeta {
				&#meta_ident
			}

			fn dispatch_key(&self) -> &'static str {
				#dispatch_key_str
			}

			fn run<'a>(
				&'a self,
				ctx: &'a SerenityContext,
				interaction: &'a CommandInteraction,
				full_command_name: &'a str,
			) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + 'a>> {
				Box::pin(async move {
					let __cmd = #struct_name {
						ctx: ctx.clone(),
						command_interaction: interaction.clone(),
						#(#extra_field_inits,)*
					};
					crate::command::command::CommandRun::#run_method(&__cmd).await
				})
			}
		}

		static #entry_static_ident: #entry_struct_ident = #entry_struct_ident;

		inventory::submit!(&#entry_static_ident as &'static dyn crate::command::registry::SlashCommand);
	};

	Ok(output)
}
