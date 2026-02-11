use proc_macro2::TokenStream;
use quote::quote;

use crate::utils::{type_to_option_type, ParamInfo};

/// Generates the code to build a `CreateCommandOption` for a single parameter.
pub fn build_option_tokens(param: &ParamInfo, fluent_prefix: &str) -> TokenStream {
	let discord_name = &param.discord_name;
	let option_type = type_to_option_type(&param.ty);
	let required = !param.is_optional;
	let autocomplete = param.autocomplete;

	// Description: use explicit #[desc] or fall back to Fluent key
	let desc_expr = if let Some(ref desc) = param.description {
		quote! { #desc.to_string() }
	} else {
		let fluent_key = format!("{}-option-{}", fluent_prefix, discord_name);
		quote! {
			{
				let lang_id = unic_langid::LanguageIdentifier::from_str("en-US").unwrap();
				loader
					.lookup(&lang_id, #fluent_key)
			}
		}
	};

	// Build choices if provided
	let choices_tokens = if !param.choices.is_empty() {
		let choice_chain: Vec<TokenStream> = param
			.choices
			.iter()
			.map(|c| {
				quote! { .add_string_choice(#c, #c) }
			})
			.collect();
		quote! { #(#choice_chain)* }
	} else {
		quote! {}
	};

	quote! {
		{
			let opt = serenity::all::CreateCommandOption::new(
				#option_type,
				#discord_name,
				#desc_expr,
			)
			.required(#required)
			.set_autocomplete(#autocomplete)
			#choices_tokens;
			opt
		}
	}
}

/// Generates code to extract a typed argument from interaction options at runtime.
pub fn build_extraction_tokens(param: &ParamInfo) -> TokenStream {
	let ident = &param.ident;
	let discord_name = &param.discord_name;

	let extraction = build_value_extraction(&param.ty, discord_name);

	if param.is_optional {
		quote! {
			let #ident = {
				let opt = opts.iter().find(|o| o.name == #discord_name);
				match opt {
					Some(o) => { #extraction },
					None => None,
				}
			};
		}
	} else {
		quote! {
			let #ident = {
				let opt = opts.iter().find(|o| o.name == #discord_name)
					.ok_or_else(|| anyhow::anyhow!("Missing required option: {}", #discord_name))?;
				let val: Option<_> = { let o = opt; #extraction };
				val.ok_or_else(|| anyhow::anyhow!("Could not resolve option: {}", #discord_name))?
			};
		}
	}
}

/// Generates the inner extraction expression that pulls a value from a `ResolvedOption`.
fn build_value_extraction(ty: &syn::Type, _name: &str) -> TokenStream {
	let type_str = get_type_ident_str(ty);

	match type_str.as_str() {
		"String" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::String(s) => Some(s.to_string()),
				_ => None,
			}
		},
		"i64" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::Integer(i) => Some(*i),
				_ => None,
			}
		},
		"f64" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::Number(n) => Some(*n),
				_ => None,
			}
		},
		"bool" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::Boolean(b) => Some(*b),
				_ => None,
			}
		},
		"User" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::User(u, _) => Some((*u).clone()),
				_ => None,
			}
		},
		"Role" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::Role(r) => Some((*r).clone()),
				_ => None,
			}
		},
		"Attachment" => quote! {
			match &o.value {
				serenity::all::ResolvedValue::Attachment(a) => Some((*a).clone()),
				_ => None,
			}
		},
		// Default: try to extract as string
		_ => quote! {
			match &o.value {
				serenity::all::ResolvedValue::String(s) => Some(s.to_string()),
				_ => None,
			}
		},
	}
}

/// Extracts the last ident of a type path as a string.
fn get_type_ident_str(ty: &syn::Type) -> String {
	if let syn::Type::Path(type_path) = ty {
		if let Some(segment) = type_path.path.segments.last() {
			return segment.ident.to_string();
		}
	}
	String::new()
}
