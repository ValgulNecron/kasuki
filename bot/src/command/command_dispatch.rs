use crate::command::admin::anilist::add_activity::AddActivityCommand;
use crate::command::admin::anilist::delete_activity::DeleteActivityCommand;
use crate::command::admin::server::lang::LangCommand;
use crate::command::admin::server::module::{check_activation_status, ModuleCommand};
use crate::command::admin::server::new_member_setting::NewMemberSettingCommand;
use crate::command::ai::image::ImageCommand;
use crate::command::ai::question::QuestionCommand;
use crate::command::ai::transcript::TranscriptCommand;
use crate::command::ai::translation::TranslationCommand;
use crate::command::anilist_server::list_all_activity::ListAllActivity;
use crate::command::anilist_server::list_register_user::ListRegisterUser;
use crate::command::anilist_user::anime::AnimeCommand;
use crate::command::anilist_user::character::CharacterCommand;
use crate::command::anilist_user::compare::CompareCommand;
use crate::command::anilist_user::level::LevelCommand;
use crate::command::anilist_user::ln::LnCommand;
use crate::command::anilist_user::manga::MangaCommand;
use crate::command::anilist_user::random::RandomCommand;
use crate::command::anilist_user::register::RegisterCommand;
use crate::command::anilist_user::search::SearchCommand;
use crate::command::anilist_user::seiyuu::SeiyuuCommand;
use crate::command::anilist_user::staff::StaffCommand;
use crate::command::anilist_user::studio::StudioCommand;
use crate::command::anilist_user::user::UserCommand;
use crate::command::anilist_user::waifu::WaifuCommand;
use crate::command::anime::random_image::AnimeRandomImageCommand;
use crate::command::anime_nsfw::random_nsfw_image::AnimeRandomNsfwImageCommand;
use crate::command::audio::join::AudioJoinCommand;
use crate::command::audio::play::AudioPlayCommand;
use crate::command::bot::credit::CreditCommand;
use crate::command::bot::info::InfoCommand;
use crate::command::bot::ping::PingCommand;
use crate::command::command_trait::SlashCommand;
use crate::command::guess_kind::guess_command_kind;
use crate::command::management::give_premium_sub::GivePremiumSubCommand;
use crate::command::management::kill_switch::KillSwitchCommand;
use crate::command::management::remove_test_sub::RemoveTestSubCommand;
use crate::command::server::generate_image_pfp_server::GenerateImagePfPCommand;
use crate::command::server::generate_image_pfp_server_global::GenerateGlobalImagePfPCommand;
use crate::command::server::guild::GuildCommand;
use crate::command::steam::steam_game_info::SteamGameInfoCommand;
use crate::command::user::avatar::AvatarCommand;
use crate::command::user::banner::BannerCommand;
use crate::command::user::command_usage::CommandUsageCommand;
use crate::command::user::profile::ProfileCommand;
use crate::command::vn::character::VnCharacterCommand;
use crate::command::vn::game::VnGameCommand;
use crate::command::vn::producer::VnProducerCommand;
use crate::command::vn::staff::VnStaffCommand;
use crate::command::vn::stats::VnStatsCommand;
use crate::command::vn::user::VnUserCommand;
use crate::config::DbConfig;
use crate::database;
use crate::database::module_activation::Model;
use crate::database::prelude::ModuleActivation;
use crate::event_handler::Handler;
use crate::get_url;
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, QueryFilter};
use serenity::all::{CommandInteraction, Context};
use std::error::Error;
use tracing::trace;

pub async fn dispatch_command(
    ctx: &Context,
    command_interaction: &CommandInteraction,
    self_handler: &Handler,
) -> Result<(), Box<dyn Error>> {
    let (kind, name) = guess_command_kind(command_interaction);

    let full_command_name = format!("{} {}", kind, name);

    trace!("Running command: {}", full_command_name);

    match name.as_str() {
        "user_avatar" => {
            AvatarCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "user_banner" => {
            BannerCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "user_profile" => {
            ProfileCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "user_command_usage" => {
            CommandUsageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                command_usage: self_handler
                    .bot_data
                    .number_of_command_use_per_command
                    .clone(),
            }
            .run_slash()
            .await?
        }

        "admin_general_lang" => {
            LangCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "admin_general_module" => {
            ModuleCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "admin_general_new_member_setting" => {
            NewMemberSettingCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "admin_anilist_add_activity" => {
            AddActivityCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "admin_anilist_delete_activity" => {
            DeleteActivityCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }

        "steam_game" => {
            SteamGameInfoCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                apps: self_handler.bot_data.apps.clone(),
            }
            .run_slash()
            .await?
        }

        "ai_image" => {
            ImageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await?
        }
        "ai_question" => {
            QuestionCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await?
        }
        "ai_transcript" => {
            TranscriptCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await?
        }
        "ai_translation" => {
            TranslationCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                handler: self_handler,
                command_name: full_command_name.clone(),
            }
            .run_slash()
            .await?
        }

        "list_user" => {
            ListRegisterUser {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "list_activity" => {
            ListAllActivity {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "anime" => {
            AnimeCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "character" => {
            CharacterCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "compare" => {
            CompareCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "level" => {
            LevelCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "ln" => {
            LnCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "manga" => {
            MangaCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "anilist_user" => {
            UserCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "waifu" => {
            WaifuCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "random" => {
            RandomCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "register" => {
            RegisterCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "staff" => {
            StaffCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "studio" => {
            StudioCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "search" => {
            SearchCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "seiyuu" => {
            SeiyuuCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                anilist_cache: self_handler.bot_data.anilist_cache.clone(),
            }
            .run_slash()
            .await?
        }

        "random_anime_random_image" => {
            AnimeRandomImageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "random_hanime_random_himage" => {
            AnimeRandomNsfwImageCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "audio_join" => {
            AudioJoinCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "audio_play" => {
            AudioPlayCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "bot_credit" => {
            CreditCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "bot_info" => {
            InfoCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "bot_ping" => {
            PingCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "kill_switch" => {
            KillSwitchCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "give_premium_sub" => {
            GivePremiumSubCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "remove_test_sub" => {
            RemoveTestSubCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "server_guild" => {
            GuildCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "server_guild_image" => {
            GenerateImagePfPCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }
        "server_guild_image_g" => {
            GenerateGlobalImagePfPCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
            }
            .run_slash()
            .await?
        }

        "vn_game" => {
            VnGameCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "vn_character" => {
            VnCharacterCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "vn_staff" => {
            VnStaffCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "vn_user" => {
            VnUserCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "vn_producer" => {
            VnProducerCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        "vn_stats" => {
            VnStatsCommand {
                ctx: ctx.clone(),
                command_interaction: command_interaction.clone(),
                config: self_handler.bot_data.config.clone(),
                vndb_cache: self_handler.bot_data.vndb_cache.clone(),
            }
            .run_slash()
            .await?
        }
        _ => {
            Err(anyhow::anyhow!("Command not found"))?;
        }
    };

    self_handler
        .increment_command_use_per_command(
            full_command_name,
            command_interaction.user.id.to_string(),
            command_interaction.user.name.to_string(),
        )
        .await;

    Ok(())
}

pub async fn check_if_module_is_on(
    guild_id: String,
    module: &str,
    db_config: DbConfig,
) -> Result<bool, Box<dyn Error>> {
    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    let row = ModuleActivation::find()
        .filter(database::module_activation::Column::GuildId.eq(guild_id.clone()))
        .one(&connection)
        .await?
        .unwrap_or(Model {
            guild_id: guild_id.clone(),
            ai_module: true,
            anilist_module: true,
            game_module: true,
            new_members_module: false,
            anime_module: true,
            vn_module: true,
            updated_at: Default::default(),
        });

    let state = check_activation_status(module, row).await;

    let state = state && check_kill_switch_status(module, db_config, guild_id).await?;

    Ok(state)
}

async fn check_kill_switch_status(
    module: &str,
    db_config: DbConfig,
    guild_id: String,
) -> Result<bool, Box<dyn Error>> {
    let connection = sea_orm::Database::connect(get_url(db_config.clone())).await?;

    let row = ModuleActivation::find()
        .filter(database::kill_switch::Column::GuildId.eq(guild_id.clone()))
        .one(&connection)
        .await?
        .unwrap_or(Model {
            guild_id,
            ai_module: true,
            anilist_module: true,
            game_module: true,
            new_members_module: false,
            anime_module: true,
            vn_module: true,
            updated_at: Default::default(),
        });

    trace!(?row);

    Ok(check_activation_status(module, row).await)
}
