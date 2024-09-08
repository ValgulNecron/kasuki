use serenity::all::{Cache, Http, Member, ShardManager};
use std::sync::Arc;
use sysinfo::System;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};
use tracing::trace;

use crate::api::grpc_server::service::info::proto::info_server::{Info, InfoServer};
use crate::api::grpc_server::service::info::proto::{
    BotInfo, BotInfoData, BotProfile, BotStat, BotSystemUsage, Guild, GuildInfo, InfoRequest,
    InfoResponse, OwnerInfo, ShardStats, SystemInfoData, TeamMember, User, UserInfo,
};
use crate::config::Config;
use crate::constant::APP_VERSION;
use crate::custom_serenity_impl::{InternalMembershipState, InternalTeamMemberRole};
use crate::event_handler::{BotData, RootUsage};

// Proto module contains the protobuf definitions for the shard service
pub(crate) mod proto {

    // Include the protobuf definitions for the shard service
    tonic::include_proto!("info");

    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const INFO_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("info_descriptor");
}

pub struct InfoService {
    pub bot_info: Arc<BotData>,
    pub sys: Arc<RwLock<System>>,
    pub command_usage: Arc<RwLock<RootUsage>>,
    pub shard_manager: Arc<ShardManager>,
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub config: Arc<Config>,
}

#[tonic::async_trait]

impl Info for InfoService {
    async fn get_info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {

        trace!("Got a info request");

        let bot_data = self.bot_info.clone();

        let guard = bot_data.bot_info.read().await.clone();

        let bot_info_data = guard.ok_or(Status::internal("Failed to get the bot info."))?;

        let sys = self.sys.clone();

        sys.write().await.refresh_all();

        let sys = sys.read().await;

        let processes = sys.processes();

        let pid = match sysinfo::get_current_pid() {
            Ok(pid) => pid,
            _ => return Err(Status::internal("Process not found.")),
        };

        let process = match processes.get(&pid) {
            Some(proc) => proc,
            _ => return Err(Status::internal("Failed to get the process.")),
        };

        let system_cpu_count = sys.cpus().len();

        // shard stats
        let shard_manager = self.shard_manager.runners.lock().await;

        let mut shard_info = Vec::new();

        for (shard_id, shard) in shard_manager.iter() {

            let id = shard_id.0.to_string();

            let latency = shard.latency.unwrap_or_default().as_millis().to_string();

            let stage = shard.stage.to_string();

            shard_info.push(ShardStats { id, latency, stage })
        }

        // bot stat
        let uptime = process.run_time();

        let uptime = format!("{}s", uptime);

        let number_of_commands_executed = self.command_usage.read().await.get_total_command_use();

        let number_of_members = self.cache.user_count() as i64;

        let number_of_guilds = self.cache.guild_count() as i64;

        let stat = Some(BotStat {
            uptime,
            number_of_commands_executed,
            number_of_members,
            number_of_guilds,
            shard_info,
        });

        // bot usage
        let cpu = format!("{}%", process.cpu_usage() / system_cpu_count as f32);

        let memory = process.memory();

        let memory = format!("{:.2}Mb", memory / 1024 / 1024);

        let usage = Some(BotSystemUsage { cpu, memory });

        // bot info
        let name = bot_info_data.name.clone();

        let version = APP_VERSION.to_string();

        let id = bot_info_data.id;

        let bot_activity = self.config.bot.bot_activity.clone();

        let description = bot_info_data.description.clone();

        let bot_data = self.http.clone().get_current_user().await;

        let id = id.to_string();

        let bot_profile: Option<BotProfile> = match bot_data {
            Ok(user) => {

                let profile_picture = user.face();

                let banner = user.banner_url();

                Some(BotProfile {
                    profile_picture,
                    banner,
                })
            }
            _ => None,
        };

        let info = Some(BotInfo {
            name,
            version,
            id,
            bot_activity,
            description,
            bot_profile,
        });

        // bot owner
        let bot_owner = match bot_info_data.owner.clone() {
            Some(owner) => owner,
            _ => return Err(Status::internal("Failed to get the bot owner.")),
        };

        let name = bot_owner.name.clone();

        let id = bot_owner.id;

        let owner_data = self.http.clone().get_user(id).await;

        let id = id.to_string();

        let team = bot_info_data.team.clone();

        trace!(?team);

        let owner_info = match (owner_data, team) {
            (Ok(user), None) => {

                let profile_picture = user.face();

                let banner = user.banner_url();

                Some(OwnerInfo {
                    name,
                    id,
                    profile_picture,
                    banner,
                    team_owned: false,
                    team_members: Vec::new(),
                    team_owner: None,
                })
            }
            (_, Some(team)) => {

                let name = team.name;

                let id = team.id.to_string();

                let icon_hash = match team.icon {
                    Some(icon) => icon.to_string(),
                    None => String::from("1"),
                };

                let profile_picture = format!(
                    "https://cdn.discordapp.com/team-icons/{}.png?size=2048",
                    icon_hash
                );

                let owner_id = team.owner_user_id;

                let mut team_members = vec![];

                let mut team_owner = None;

                for member in team.members {

                    let owner_id = owner_id.to_string();

                    let role: InternalTeamMemberRole = member.role.into();

                    let membership_state: InternalMembershipState = member.membership_state.into();

                    let user = member.user;

                    let username = user.name.clone();

                    let id = user.id.to_string();

                    let profile_picture = user.face();

                    let banner = user.banner_url();

                    trace!(id);

                    trace!(owner_id);

                    if id == owner_id {

                        team_owner = Some(TeamMember {
                            role: role.to_string(),
                            membership_state: membership_state.to_string(),
                            username,
                            id,
                            profile_picture,
                            banner,
                        });

                        continue;
                    }

                    team_members.push(TeamMember {
                        role: role.to_string(),
                        membership_state: membership_state.to_string(),
                        username,
                        id,
                        profile_picture,
                        banner,
                    });
                }

                let owner_info = OwnerInfo {
                    name,
                    id,
                    profile_picture,
                    banner: None,
                    team_owned: true,
                    team_members,
                    team_owner,
                };

                Some(owner_info)
            }
            _ => None,
        };

        trace!("Owner info: {:?}", bot_owner);

        let guild_count = self.cache.guild_count() as i64;

        let mut guilds = Vec::new();

        let mut members: Vec<Member> = Vec::new();

        for guild in self.cache.guilds() {

            let real_guild = match guild.to_partial_guild_with_counts(&self.http).await {
                Ok(guild) => guild,
                _ => continue,
            };

            let id = real_guild.id.clone().to_string();

            let name = real_guild.name.clone();

            let owner_id = real_guild.owner_id.clone().to_string();

            let icon = real_guild.icon_url();

            let banner = real_guild.banner_url();

            let description = real_guild.description.clone();

            let mut members_temp = real_guild
                .members(&self.http, Some(1000), None)
                .await
                .unwrap_or_default();

            members.append(&mut members_temp.clone());

            while members_temp.len() > 1000 {

                members_temp = real_guild
                    .members(
                        &self.http,
                        Some(1000),
                        Some(match members_temp.last() {
                            Some(m) => m.user.id,
                            None => break,
                        }),
                    )
                    .await
                    .unwrap_or_default();

                members.append(&mut members_temp.clone());
            }

            let guild = Guild {
                id,
                name,
                owner_id,
                icon,
                banner,
                description,
            };

            guilds.push(guild);
        }

        let guild_info = Some(GuildInfo {
            guild_count,
            guilds,
        });

        // removed all duplicate members that have the same user id
        members.sort_by(|a, b| a.user.id.cmp(&b.user.id));

        let user_count = self.cache.user_count() as i64;

        let mut users = Vec::new();

        for member in members {

            let user = match member.user.id.to_user(&self.http).await {
                Ok(user) => user,
                _ => continue,
            };

            let id = user.id.clone().to_string();

            let username = user.name.clone();

            let profile_picture = user.face();

            let banner = user.banner_url();

            let is_bot = user.bot;

            let mut guilds = vec![member.guild_id.to_string()];

            let flags = user.flags;

            let mut user_flags = Vec::new();

            // If there are, iterate over the flags and add them to a vector
            for (flag, _) in flags.iter_names() {

                user_flags.push(flag)
            }

            let mut user = User {
                username,
                id,
                profile_picture,
                is_bot,
                guilds: guilds.clone(),
                banner,
            };

            // check if a User with the same id already exists and if so get the index
            let contain = users.iter().any(|u: &User| u.id == user.id);

            if contain {

                let index = users
                    .iter()
                    .position(|u| u.id == user.id)
                    .unwrap_or_default();

                let temps = users.clone();

                let user2 = match temps.get(index) {
                    Some(user) => user.clone(),
                    None => continue,
                };

                users.remove(index);

                user2.guilds.clone().append(&mut guilds);

                trace!(?user);

                trace!(?user2);

                user = user2.clone();

                trace!(?user);
            }

            users.push(user);
        }

        let user_info = Some(UserInfo { user_count, users });

        let bot_info = BotInfoData {
            stat,
            usage,
            info,
            owner_info,
            guild_info,
            user_info,
        };

        // system info
        let os = format!(
            "{}, {} {} {}",
            System::name().unwrap_or_default(),
            System::kernel_version().unwrap_or_default(),
            System::os_version().unwrap_or_default(),
            System::host_name().unwrap_or_default(),
        );

        let system_total_memory = format!("{}Gb", sys.total_memory() / 1024 / 1024 / 1024);

        let system_used_memory = format!("{}Gb", sys.used_memory() / 1024 / 1024 / 1024);

        let system_cpu_usage = format!("{}%", sys.global_cpu_usage());

        let system_cpu_name = sys.cpus()[0].name().to_string();

        let system_cpu_brand = sys.cpus()[0].brand().to_string();

        let system_cpu_frequency = sys.cpus()[0].frequency().to_string();

        let system_cpu_count = system_cpu_count.to_string();

        let sys_info = SystemInfoData {
            os,
            system_total_memory,
            system_used_memory,
            system_cpu_usage,
            system_cpu_name,
            system_cpu_brand,
            system_cpu_frequency,
            system_cpu_count,
        };

        let info_response = InfoResponse {
            bot_info: Option::from(bot_info),
            sys_info: Option::from(sys_info),
        };

        trace!("Completed a info request");

        Ok(Response::new(info_response))
    }
}

pub fn get_info_server(info_service: InfoService) -> InfoServer<InfoService> {

    InfoServer::new(info_service)
}
