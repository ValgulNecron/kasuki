// Proto module contains the protobuf definitions for the shard service
pub(crate) mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("info");
    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const INFO_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("info_descriptor");
}

use crate::constant::{ACTIVITY_NAME, APP_VERSION};
use crate::grpc_server::service::info::proto::info_server::{Info, InfoServer};
use crate::grpc_server::service::info::proto::{BotInfo, BotInfoData, BotProfile, BotStat, BotSystemUsage, InfoRequest, InfoResponse, OwnerInfo, ShardStats, SystemInfoData};
use serenity::all::{Cache, CurrentApplicationInfo, ShardManager};
use std::sync::{Arc};
use sysinfo::System;
use tokio::sync::{RwLock};
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct InfoService {
    pub bot_info: Arc<CurrentApplicationInfo>,
    pub sys: Arc<RwLock<System>>,
    pub os_info: Arc<os_info::Info>,
    pub command_usage: Arc<RwLock<u128>>,
    pub shard_manager: Arc<ShardManager>,
    pub cache: Arc<Cache>,
}

#[tonic::async_trait]
impl Info for InfoService {
    async fn get_info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        trace!("Got a info request");
        let bot_info_data = self.bot_info.clone();
        let sys = self.sys.clone();
        let os_info = self.os_info.clone();
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

        // shard stats
        let shard_manager = self.shard_manager.runners.lock().await;
        let mut shard_info = Vec::new();
        for (shard_id, shard) in shard_manager.iter() {
            let id = shard_id.0.to_string();
            let latency = shard.latency.unwrap_or_default().as_millis().to_string();
            let stage = shard.stage.to_string();
            shard_info.push(
                ShardStats {
                    id,
                    latency,
                    stage,
                }
            )
        }


        // bot stat
        let uptime = process.run_time();
        let uptime = format!("{}s", uptime);
        let command_usage_guard = self.command_usage.read().await;
let number_of_commands_executed: u128 = *command_usage_guard;
let number_of_commands_executed = number_of_commands_executed as i64;
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
        let cpu = format!("{}%", process.cpu_usage());
        let memory = process.memory();
        let memory = format!("{:.2}Mb", memory / 1024 / 1024);
        let usage = Some(BotSystemUsage {
            cpu,
            memory,
        });

        // bot info
        let name = bot_info_data.name.clone();
        let version = APP_VERSION.to_string();
        let id = bot_info_data.id;
        let bot_activity = ACTIVITY_NAME.to_string();
        let description = bot_info_data.description.clone();
        let bot_data = self.cache.user(id.get());
        let id = id.to_string();
        let bot_profile: Option<BotProfile> = match bot_data {
            Some(user) => {
                let profile_picture = user.face();
                let banner = user.banner_url();
                Some(BotProfile {
                    profile_picture,
                    banner,
                })
            },
            None => None
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
        let id = bot_owner.id.to_string();
        let profile_picture = bot_owner.face();
        let banner = bot_owner.banner_url();
        let owner_info = Some(OwnerInfo {
            name,
            id,
            profile_picture,
            banner,
        });

        let bot_info = BotInfoData {
            stat,
            usage,
            info,
            owner_info,
        };

        // system info
        let os = format!(
            "{}, {} {} {} {} {}",
            os_info.os_type(),
            os_info.bitness(),
            os_info.version(),
            os_info.codename().unwrap_or_default(),
            os_info.architecture().unwrap_or_default(),
            os_info.edition().unwrap_or_default()
        );
        let system_total_memory = format!("{}Gb", sys.total_memory() / 1024 / 1024 / 1024);
        let system_used_memory = format!("{}Gb", sys.used_memory() / 1024 / 1024 / 1024);
        let system_cpu_usage = format!("{}%", sys.global_cpu_info().cpu_usage());
        let system_cpu_name = sys.global_cpu_info().name().to_string();
        let system_cpu_brand =sys.global_cpu_info().brand().to_string();
        let system_cpu_frequency = sys.global_cpu_info().frequency().to_string();
        let system_cpu_count = sys.cpus().len().to_string();
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
