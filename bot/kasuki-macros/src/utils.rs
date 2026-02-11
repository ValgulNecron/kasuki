use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{Attribute, Expr, FnArg, Ident, Lit, Meta, Pat, PatType, Type};

/// Parsed command-level attributes from `#[command(...)]` or `#[command_group(...)]`.
#[derive(Default)]
pub struct CommandAttrs {
	pub nsfw: bool,
	pub guild_only: bool,
	pub contexts: Vec<Ident>,
	pub permissions: Vec<Ident>,
}

impl CommandAttrs {
	pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
		let mut result = Self::default();

		for attr in attrs {
			let path = attr.path();
			if !(path.is_ident("command") || path.is_ident("command_group")) {
				continue;
			}

			if let Meta::List(_) = &attr.meta {
				attr.parse_nested_meta(|meta| {
					if meta.path.is_ident("nsfw") {
						result.nsfw = true;
						Ok(())
					} else if meta.path.is_ident("guild_only") {
						result.guild_only = true;
						Ok(())
					} else if meta.path.is_ident("contexts") {
						let content;
						syn::bracketed!(content in meta.input);
						let idents = content
							.parse_terminated(Ident::parse, syn::Token![,])?;
						result.contexts = idents.into_iter().collect();
						Ok(())
					} else if meta.path.is_ident("permissions") {
						let content;
						syn::bracketed!(content in meta.input);
						let idents = content
							.parse_terminated(Ident::parse, syn::Token![,])?;
						result.permissions = idents.into_iter().collect();
						Ok(())
					} else {
						Err(meta.error("unknown command attribute"))
					}
				})?;
			}
		}

		Ok(result)
	}
}

/// Information about a single function parameter (Discord command option).
pub struct ParamInfo {
	pub ident: Ident,
	pub discord_name: String,
	pub ty: Type,
	pub is_optional: bool,
	pub autocomplete: bool,
	pub choices: Vec<String>,
	pub description: Option<String>,
}

/// Attributes that can appear on function parameters.
struct ParamAttrs {
	autocomplete: bool,
	choices: Vec<String>,
	desc: Option<String>,
	name: Option<String>,
}

fn parse_param_attrs(attrs: &[Attribute]) -> syn::Result<ParamAttrs> {
	let mut result = ParamAttrs {
		autocomplete: false,
		choices: Vec::new(),
		desc: None,
		name: None,
	};

	for attr in attrs {
		let path = attr.path();

		if path.is_ident("autocomplete") {
			result.autocomplete = true;
		} else if path.is_ident("choices") {
			if let Meta::List(list) = &attr.meta {
				let tokens = list.tokens.clone();
				let parsed: syn::punctuated::Punctuated<Expr, syn::Token![,]> =
					syn::parse::Parser::parse2(
						syn::punctuated::Punctuated::parse_terminated,
						tokens,
					)?;

				for expr in parsed {
					let choice = match &expr {
						Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
						Expr::Lit(lit) => match &lit.lit {
							Lit::Str(s) => Some(s.value()),
							_ => None,
						},
						_ => None,
					};
					if let Some(c) = choice {
						result.choices.push(c);
					}
				}
			}
		} else if path.is_ident("desc") {
			if let Meta::NameValue(nv) = &attr.meta {
				if let Expr::Lit(lit) = &nv.value {
					if let Lit::Str(s) = &lit.lit {
						result.desc = Some(s.value());
					}
				}
			}
		} else if path.is_ident("name") {
			if let Meta::NameValue(nv) = &attr.meta {
				if let Expr::Lit(lit) = &nv.value {
					if let Lit::Str(s) = &lit.lit {
						result.name = Some(s.value());
					}
				}
			}
		}
	}

	Ok(result)
}

/// Extracts `ParamInfo` from function parameters, skipping `ctx` and `interaction`.
pub fn extract_params(inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) -> syn::Result<Vec<ParamInfo>> {
	let mut params = Vec::new();

	for arg in inputs.iter().skip(2) {
		// Skip &self, ctx, and interaction
		let FnArg::Typed(PatType { pat, ty, attrs, .. }) = arg else {
			continue;
		};

		let Pat::Ident(pat_ident) = pat.as_ref() else {
			continue;
		};

		let param_attrs = parse_param_attrs(attrs)?;
		let ident = pat_ident.ident.clone();
		let discord_name = param_attrs
			.name
			.unwrap_or_else(|| ident.to_string());

		let (is_optional, inner_ty) = unwrap_option_type(ty);

		params.push(ParamInfo {
			ident,
			discord_name,
			ty: inner_ty,
			is_optional,
			autocomplete: param_attrs.autocomplete,
			choices: param_attrs.choices,
			description: param_attrs.desc,
		});
	}

	Ok(params)
}

/// If the type is `Option<T>`, returns `(true, T)`. Otherwise `(false, original)`.
fn unwrap_option_type(ty: &Type) -> (bool, Type) {
	if let Type::Path(type_path) = ty {
		if let Some(segment) = type_path.path.segments.last() {
			if segment.ident == "Option" {
				if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
					if let Some(syn::GenericArgument::Type(inner)) = args.args.first() {
						return (true, inner.clone());
					}
				}
			}
		}
	}
	(false, ty.clone())
}

/// Generates the `CommandOptionType` token for a Rust type.
pub fn type_to_option_type(ty: &Type) -> TokenStream {
	if let Type::Path(type_path) = ty {
		if let Some(segment) = type_path.path.segments.last() {
			let ident_str = segment.ident.to_string();
			return match ident_str.as_str() {
				"String" | "str" => quote! { serenity::all::CommandOptionType::String },
				"i64" | "i32" | "i16" | "i8" | "u64" | "u32" | "u16" | "u8" => {
					quote! { serenity::all::CommandOptionType::Integer }
				}
				"f64" | "f32" => quote! { serenity::all::CommandOptionType::Number },
				"bool" => quote! { serenity::all::CommandOptionType::Boolean },
				"User" => quote! { serenity::all::CommandOptionType::User },
				"Channel" | "GuildChannel" | "PartialChannel" => {
					quote! { serenity::all::CommandOptionType::Channel }
				}
				"Role" => quote! { serenity::all::CommandOptionType::Role },
				"Attachment" => {
					quote! { serenity::all::CommandOptionType::Attachment }
				}
				_ => quote! { serenity::all::CommandOptionType::String },
			};
		}
	}
	quote! { serenity::all::CommandOptionType::String }
}

/// Strips command-specific attributes (`#[autocomplete]`, `#[choices(...)]`,
/// `#[desc = "..."]`, `#[name = "..."]`) from function parameters so the
/// resulting function compiles without unknown attribute errors.
pub fn strip_command_attrs(inputs: &mut syn::punctuated::Punctuated<FnArg, syn::Token![,]>) {
	let command_attr_names = ["autocomplete", "choices", "desc", "name"];

	for arg in inputs.iter_mut() {
		if let FnArg::Typed(pat_type) = arg {
			pat_type.attrs.retain(|attr| {
				let path = attr.path();
				!command_attr_names
					.iter()
					.any(|name| path.is_ident(name))
			});
		}
	}
}
