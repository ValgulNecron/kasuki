use std::error::Error;
use std::sync::Arc;

use crate::command::command_trait::{Command, SlashCommand, UserCommand};
use crate::command::user::avatar::{get_user_command, get_user_command_user};
use crate::config::Config;
use crate::helper::create_default_embed::get_default_embed;
use crate::helper::error_management::error_enum::ResponseError;
use crate::structure::message::user::profile::{load_localization_profile, ProfileLocalised};
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseMessage,
    EntitlementKind, Member, User,
};

pub struct ProfileCommand {
    pub ctx: Context,
    pub command_interaction: CommandInteraction,
    pub config: Arc<Config>,
}

impl Command for ProfileCommand {
    fn get_ctx(&self) -> &Context {
        &self.ctx
    }
    fn get_command_interaction(&self) -> &CommandInteraction {
        &self.command_interaction
    }
}

impl SlashCommand for ProfileCommand {
    async fn run_slash(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command(&self.ctx, &self.command_interaction).await?;
        send_embed(&self.ctx, &self.command_interaction, user, &self.config).await
    }
}

impl UserCommand for ProfileCommand {
    async fn run_user(&self) -> Result<(), Box<dyn Error>> {
        let user = get_user_command_user(&self.ctx, &self.command_interaction).await;
        send_embed(&self.ctx, &self.command_interaction, user, &self.config).await
    }
}

fn get_fields(profile_localised: &ProfileLocalised, user: User) -> Vec<(String, String, bool)> {
    let mut fields = vec![
        (
            profile_localised.id.clone(),
            user.id.clone().to_string(),
            true,
        ),
        (
            profile_localised.creation_date.clone(),
            format!("<t:{}>", user.created_at().timestamp()),
            true,
        ),
        (profile_localised.bot.clone(), user.bot.to_string(), true),
        (
            profile_localised.system.clone(),
            user.system.to_string(),
            true,
        ),
    ];
    if let Some(public_flag) = user.public_flags {
        let mut user_flags = Vec::new();
        // If there are, iterate over the flags and add them to a vector
        for (flag, _) in public_flag.iter_names() {
            user_flags.push(flag)
        }
        if !user_flags.is_empty() {
            fields.push((
                profile_localised.public_flag.clone(),
                user_flags.join(" / "),
                false,
            ));
        }
    }

    fields
}

async fn send_embed(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    user: User,
    config: &Arc<Config>,
) -> Result<(), Box<dyn Error>> {
    let db_type = config.bot.config.db_type.clone();
    let db_config = config.bot.config.clone();
    let guild_id = command_interaction
        .guild_id
        .map(|id| id.to_string())
        .unwrap_or("0".to_string());
    let profile_localised = load_localization_profile(guild_id, db_config).await?;
    let mut fields = get_fields(&profile_localised, user.clone());
    let avatar_url = user.face();

    let member: Option<Member> = {
        match command_interaction.guild_id {
            Some(guild_id) => match guild_id.member(&ctx.http, user.id).await {
                Ok(member) => Some(member),
                Err(_) => None,
            },
            None => None,
        }
    };
    if let Some(member) = member {
        if let Some(joined_at) = member.joined_at {
            fields.push((
                profile_localised.joined_date,
                format!("<t:{}>", joined_at.timestamp()),
                true,
            ));
        }
    }

    let skus = ctx.http.get_skus().await;
    let user_premium = ctx
        .http
        .get_entitlements(Some(user.id), None, None, None, None, None, Some(true))
        .await;
    if user_premium.is_ok() && skus.is_ok() {
        let skus = skus.unwrap().clone();
        let data = user_premium.unwrap();
        if !data.is_empty() {
            let string = data.iter().map(|e| {
                let sku_id = e.sku_id;
                let sku = skus.iter().find(|e2| e2.id == sku_id);
                let e_type = e.kind.clone();
                let type_name = match e_type {
                    EntitlementKind::ApplicationSubscription => {
                        String::from("APPLICATION_SUBSCRIPTION")
                    }
                    EntitlementKind::Unknown(1) => String::from("PURCHASE"),
                    EntitlementKind::Unknown(2) => String::from("PREMIUM_SUBSCRIPTION"),
                    EntitlementKind::Unknown(3) => String::from("DEVELOPER_GIFT"),
                    EntitlementKind::Unknown(4) => String::from("TEST_MODE_PURCHASE"),
                    EntitlementKind::Unknown(5) => String::from("FREE_PURCHASE"),
                    EntitlementKind::Unknown(6) => String::from("USER_GIFT"),
                    EntitlementKind::Unknown(7) => String::from("PREMIUM_PURCHASE"),
                    _ => String::from("Unknown"),
                };

                let sku_name = match sku {
                    Some(sku) => sku.name.clone(),
                    None => String::from("Unknown"),
                };
                format!(
                    "{}: {}/{} \n {}",
                    sku_name,
                    e.starts_at.unwrap_or_default(),
                    e.ends_at.unwrap_or_default(),
                    type_name
                )
            });
            fields.push((profile_localised.premium, string.collect::<String>(), true));
        }
    }

    let mut builder_embed = get_default_embed(None)
        .thumbnail(avatar_url)
        .title(
            profile_localised
                .title
                .replace("$user$", user.name.as_str()),
        )
        .fields(fields);

    if let Some(banner) = user.banner_url() {
        builder_embed = builder_embed.image(banner);
    }

    let builder_message = CreateInteractionResponseMessage::new().embed(builder_embed);
    let builder = CreateInteractionResponse::Message(builder_message);

    command_interaction
        .create_response(&ctx.http, builder)
        .await
        .map_err(|e| ResponseError::Sending(format!("{:#?}", e)))?;
    Ok(())
}