use serenity::all::{
    AutocompleteChoice, CommandInteraction, Context, CreateAutocompleteResponse,
    CreateInteractionResponse, Sku, SkuKind,
};
use small_fixed_array::FixedString;
use tracing::debug;

use crate::constant::DEFAULT_STRING;
use crate::helper::get_option::command::get_option_map_string;

pub async fn give_premium_sub_autocomplete(
    ctx: Context,
    autocomplete_interaction: CommandInteraction,
) {
    let map = get_option_map_string(&autocomplete_interaction);

    let _subscription = map
        .get(&FixedString::from_str_trunc("subscription"))
        .unwrap_or(DEFAULT_STRING);

    let sku_list = ctx.http.get_skus().await.unwrap();

    let sku_list = sku_list
        .iter()
        .map(|sku| {
            (
                {
                    let kind = match sku.kind.0 {
                        5 => String::from("Subscription"),
                        6 => String::from("Subscription Group"),
                        2 => String::from("DURABLE"),
                        3 => String::from("CONSUMABLE"),
                        _ => String::from("Unknown"),
                    };

                    let available = 1 << 2;

                    let guild_subscription = 1 << 7;

                    let user_subscription = 1 << 8;

                    let flags_bits = sku.flags.bits();

                    let mut flags = String::new();

                    let mut flags2 = String::new();

                    if (available & flags_bits) != 0 {
                        flags2.push_str("is available ");
                    }

                    if (guild_subscription & flags_bits) != 0 {
                        flags.push_str("guild subscription ");
                    }

                    if (user_subscription & flags_bits) != 0 {
                        flags.push_str("user subscription ");
                    }

                    format!("{} {} for {} {}", sku.name, kind, flags, flags2)
                        .chars()
                        .take(100)
                        .collect::<String>()
                },
                sku.id.to_string(),
            )
        })
        .filter(|sku| !sku.0.contains("Subscription Group"))
        .map(|sku| AutocompleteChoice::new(sku.0, sku.1))
        .take(25)
        .collect::<Vec<AutocompleteChoice>>();

    if sku_list.is_empty() {
        return;
    }

    let data = CreateAutocompleteResponse::new().set_choices(sku_list);

    let builder = CreateInteractionResponse::Autocomplete(data);

    if let Err(why) = autocomplete_interaction
        .create_response(&ctx.http, builder)
        .await
    {
        debug!("Error sending response: {:?}", why);
    }
}
