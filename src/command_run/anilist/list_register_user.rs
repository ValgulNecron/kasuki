use image::{ImageBuffer, Rgba};
use serenity::all::{CommandInteraction, Context, User};
use text_to_png::TextRenderer;

use crate::anilist_struct::run::user::UserWrapper;
use crate::constant::OPTION_ERROR;
use crate::error_enum::AppError;
use crate::lang_struct::anilist::list_register_user::load_localization_list_user;
use crate::sqls::general::data::get_registered_user;

pub async fn run(ctx: &Context, command: &CommandInteraction) -> Result<(), AppError> {
    let guild_id = match command.guild_id {
        Some(id) => id.to_string(),
        None => String::from("0"),
    };

    let list_user_localised = load_localization_list_user(guild_id).await?;

    let guild_id = command.guild_id.ok_or(OPTION_ERROR.clone())?;

    let guild = guild_id
        .to_partial_guild_with_counts(&ctx.http)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let members = guild
        .members(&ctx.http, Some(1000u64), None)
        .await
        .map_err(|_| OPTION_ERROR.clone())?;

    let mut anilist_user = Vec::new();
    for member in members {
        let user_id = member.user.id.to_string();
        let row: (Option<String>, Option<String>) = get_registered_user(&user_id).await?;
        let user_date = UserWrapper::new_user_by_id(row.1.unwrap().parse::<i32>().unwrap()).await?;
        let data = Data {
            user: member.user,
            anilist: user_date,
        };
        anilist_user.push(data)
    }

    let text = anilist_user
        .iter()
        .map(|user| {
            format!(
                "{}: {}",
                user.user.name,
                user.anilist.data.user.name.clone().unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let renderer = TextRenderer::default();
    let text_png = renderer.render_text_to_png_data(text, 64, "Black").unwrap();

    let img = ImageBuffer::from_fn(text_png.size.width, text_png.size.height, |x, y| {
        let pixel = &text_png.data[((y * text_png.size.width + x) * 4) as usize..];
        Rgba([pixel[0], pixel[1], pixel[2], pixel[3]])
    });

    // Save the image
    img.save("output.png").unwrap();

    Ok(())
}

struct Data {
    pub user: User,
    pub anilist: UserWrapper,
}
