use crate::helper::create_default_embed::get_default_embed;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use tracing::trace;

use crate::helper::error_management::error_enum::{AppError, ErrorResponseType, ErrorType};
use crate::helper::get_option::subcommand::get_option_map_string_subcommand;
use crate::helper::vndbapi::game::get_vn;

pub async fn run(ctx: &Context, command_interaction: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command_interaction.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };
    let map = get_option_map_string_subcommand(command_interaction);
    trace!("{:?}", map);
    let game = map
        .get(&String::from("title"))
        .cloned()
        .unwrap_or(String::new());
    let vn = get_vn(game.clone()).await?;
    let vn = vn.results[0].clone();
    let mut fields = vec![];
    fields.push((String::from("ID"), vn.id.clone(), true));
    fields.push((String::from("Title"), vn.title.clone(), true));

    let mut builder_embed = get_default_embed(None)
        .description(vn.description.unwrap_or_default().clone())
        .fields(fields);
    let sexual = match vn.image.clone() {
        Some(image) => image.sexual,
        None => 2.0,
    };
    let violence = match vn.image.clone() {
        Some(image) => image.violence,
        None => 2.0,
    };
    let url: Option<String> = match vn.image {
        Some(image) => Some(image.url.clone()),
        None => None,
    };
    if (sexual <= 1.5) && (violence <= 1.0) {
        if let Some(url) = url {
            builder_embed = builder_embed.image(url);
        }
    }
    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);
    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| {
            AppError::new(
                format!("Error while sending the command {}", e),
                ErrorType::Command,
                ErrorResponseType::Message,
            )
        })
}
