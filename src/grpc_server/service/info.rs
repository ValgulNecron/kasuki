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
use crate::grpc_server::service::info::proto::{
    BotInfoData, InfoRequest, InfoResponse, SystemInfoData,
};
use serenity::all::CurrentApplicationInfo;
use std::sync::{Arc, RwLock};
use sysinfo::System;
use tonic::{Request, Response, Status};
use tracing::trace;

pub struct InfoService {
    pub bot_info: Arc<CurrentApplicationInfo>,
    pub sys: Arc<RwLock<System>>,
    pub os_info: Arc<os_info::Info>,
}

#[tonic::async_trait]
impl Info for InfoService {
    async fn get_info(
        &self,
        _request: Request<InfoRequest>,
    ) -> Result<Response<InfoResponse>, Status> {
        trace!("Got a info request");
        let bot_info = self.bot_info.clone();
        let sys = self.sys.clone();
        let info = self.os_info.clone();
        match sys.write() {
            Ok(mut guard) => guard.refresh_all(),
            _ => {}
        }

        let sys = sys.read().unwrap();
        let processes = sys.processes();
        let pid = match sysinfo::get_current_pid() {
            Ok(pid) => pid,
            _ => return Err(Status::internal("Process not found.")),
        };
        let process = match processes.get(&pid) {
            Some(proc) => proc,
            _ => return Err(Status::internal("Failed to get the process.")),
        };

        // system info
        let os = format!(
            "{}, {} {} {} {} {}",
            info.os_type(),
            info.bitness(),
            info.version(),
            info.codename().unwrap_or_default(),
            info.architecture().unwrap_or_default(),
            info.edition().unwrap_or_default()
        );
        let system_total_memory = format!("{}Gb", sys.total_memory() / 1024 / 1024 / 1024);
        let system_used_memory = format!("{}Gb", sys.used_memory() / 1024 / 1024 / 1024);
        let system_cpu_usage = format!("{}%", sys.global_cpu_info().cpu_usage());

        let app_cpu = format!("{}%", process.cpu_usage());
        let app_memory = process.memory();
        let app_memory = format!("{:.2}Mb", app_memory / 1024 / 1024);

        // bot info
        let bot_name = bot_info.name.clone();
        let version = APP_VERSION.to_string();
        let bot_id = bot_info.id.to_string();
        let bot_owner = match bot_info.owner.clone() {
            Some(owner) => owner.name,
            _ => return Err(Status::internal("Failed to get the bot owner.")),
        };
        let bot_activity = ACTIVITY_NAME.to_string();
        let description = bot_info.description.clone();
        let uptime = process.run_time();
        let bot_uptime = format!("{}s", uptime);

        let bot_info_data = BotInfoData {
            bot_name,
            version,
            bot_uptime,
            bot_id,
            bot_owner,
            bot_activity,
            description,
        };

        let sys_info_data = SystemInfoData {
            app_cpu,
            app_memory,
            os,
            system_total_memory,
            system_used_memory,
            system_cpu_usage,
        };

        let info_response = InfoResponse {
            bot_info: Option::from(bot_info_data),
            sys_info: Option::from(sys_info_data),
        };
        trace!("Completed a info request");
        Ok(Response::new(info_response))
    }
}

pub fn get_info_server(info_service: InfoService) -> InfoServer<InfoService> {
    InfoServer::new(info_service)
}
