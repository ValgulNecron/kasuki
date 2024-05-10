use std::sync::{Arc, RwLock};

use serenity::all::{CurrentApplicationInfo, ShardId, ShardManager};
use sysinfo::System;
use tonic::{Request, Response, Status};
use tracing::trace;

use proto::shard_server::Shard;

use crate::constant::{
    ACTIVITY_NAME, APP_VERSION, BOT_INFO, GRPC_CERT_PATH, GRPC_KEY_PATH, GRPC_SERVER_PORT,
    GRPC_USE_TLS,
};
use crate::grpc_server::launcher::proto::info_server::{Info, InfoServer};
use crate::grpc_server::launcher::proto::shard_server::ShardServer;
use crate::grpc_server::launcher::proto::{BotInfoData, InfoRequest, InfoResponse, SystemInfoData};

// Proto module contains the protobuf definitions for the shard service
mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("shard");
    tonic::include_proto!("info");
    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const SHARD_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("shard_descriptor");
    pub(crate) const INFO_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("info_descriptor");
}

// ShardService is a struct that contains a reference to the ShardManager
#[derive(Debug)]
struct ShardService {
    // shard_manager is an Arc<ShardManager> that manages the shards
    pub shard_manager: Arc<ShardManager>,
}

// Shard is a trait that defines the methods for the shard service
#[tonic::async_trait]
impl Shard for ShardService {
    // shard_count is an async function that returns the count of shards and their ids
    // It takes a Request<proto::ShardCountRequest> as a parameter and returns a Result<Response<proto::ShardCountResponse>, Status>
    async fn shard_count(
        &self,
        _request: Request<proto::ShardCountRequest>,
    ) -> Result<Response<proto::ShardCountResponse>, Status> {
        trace!("Got a shard count request");
        // Clone the shard_manager
        let shard_manager = self.shard_manager.clone();
        // Initialize an empty vector for the shard ids
        let mut shard_ids = Vec::new();
        // Iterate over the shard runners and push their ids to the shard_ids vector
        for (shard_id, _) in shard_manager.runners.lock().await.iter() {
            shard_ids.push(shard_id.0 as i32);
        }
        // Create a ShardCountResponse with the count of shards and their ids
        let reply = proto::ShardCountResponse {
            count: shard_ids.len() as i32,
            shard_ids,
        };
        trace!("Completed a shard count request");
        // Return the ShardCountResponse
        Ok(Response::new(reply))
    }

    // shard_info is an async function that returns the information of a specific shard
    // It takes a Request<proto::ShardInfoRequest> as a parameter and returns a Result<Response<proto::ShardInfoResponse>, Status>
    async fn shard_info(
        &self,
        request: Request<proto::ShardInfoRequest>,
    ) -> Result<Response<proto::ShardInfoResponse>, Status> {
        trace!("Got a shard info request");
        // Get the data from the request
        let data = request.into_inner();
        // Get the id of the shard
        let id = data.shard_id;
        // Clone the shard_manager
        let shard_manager = self.shard_manager.clone();
        // Lock the shard runners
        let runners = shard_manager.runners.lock().await;
        // If the shard is not found, return an error
        if !runners.contains_key(&ShardId(id as u32)) {
            return Err(Status::not_found("Shard not found"));
        }
        // Get the shard
        let shard = runners.get(&ShardId(id as u32)).unwrap();
        // Create a ShardInfoResponse with the shard id, latency, and stage
        let reply = proto::ShardInfoResponse {
            shard_id: id,
            latency: shard.latency.unwrap_or_default().as_millis().to_string(),
            stage: shard.stage.to_string(),
        };
        trace!("Completed a shard info request");
        // Return the ShardInfoResponse
        Ok(Response::new(reply))
    }
}

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

/// `grpc_server_launcher` is an asynchronous function that launches the gRPC server for the shard service.
/// It takes a reference to an `Arc<ShardManager>` as a parameter.
/// It does not return a value.
///
/// # Arguments
///
/// * `shard_manager` - A reference to an Arc<ShardManager> that manages the shards.
///
/// # Panics
///
/// This function will panic if it fails to build the reflection service or if it fails to serve the gRPC server.
pub async fn grpc_server_launcher(shard_manager: &Arc<ShardManager>) {
    // Clone the Arc<ShardManager>
    let shard_manager_arc: Arc<ShardManager> = shard_manager.clone();

    // Define the address for the gRPC server
    let addr = format!("0.0.0.0:{}", *GRPC_SERVER_PORT);
    // Create a new ShardService with the cloned Arc<ShardManager>
    let shard_service = ShardService {
        shard_manager: shard_manager_arc,
    };
    let info_service = unsafe {
        InfoService {
            bot_info: Arc::new(BOT_INFO.clone().unwrap()),
            sys: Arc::new(RwLock::new(System::new_all())),
            os_info: Arc::new(os_info::get()),
        }
    };

    // Configure the reflection service and register the file descriptor set for the shard service
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::SHARD_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(proto::INFO_FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    let is_tls = *GRPC_USE_TLS;
    trace!("TLS: {}", is_tls);
    if is_tls {
        generate_key();
        let private_key_path = GRPC_KEY_PATH.clone();
        let cert_path = GRPC_CERT_PATH.clone();
        // Load the server's key and certificate
        let key = tokio::fs::read(private_key_path).await.unwrap();
        let cert = tokio::fs::read(cert_path).await.unwrap();
        // Convert to a string
        let key = String::from_utf8(key).unwrap();
        let cert = String::from_utf8(cert).unwrap();
        trace!("Key: {:?}", key);
        trace!("Cert: {:?}", cert);
        // Build the gRPC server with TLS, add the ShardService and the reflection service, and serve the gRPC server
        let identity = tonic::transport::Identity::from_pem(cert, key);
        trace!("Identity: {:?}", identity);
        let tls_config = tonic::transport::ServerTlsConfig::new().identity(identity);
        tonic::transport::Server::builder()
            .tls_config(tls_config)
            .unwrap()
            .add_service(ShardServer::new(shard_service))
            .add_service(InfoServer::new(info_service))
            .add_service(reflection)
            .serve(addr.parse().unwrap())
            .await
            .unwrap();
    } else {
        // Build the gRPC server, add the ShardService and the reflection service, and serve the gRPC server
        tonic::transport::Server::builder()
            .add_service(ShardServer::new(shard_service))
            .add_service(InfoServer::new(info_service))
            .add_service(reflection)
            .serve(addr.parse().unwrap())
            .await
            .unwrap();
    }
}

fn generate_key() {
    // Specify the subject alternative names. Since we're not using a domain,
    // we'll just use "localhost" as an example.
    let subject_alt_names = vec![
        "127.0.0.1".to_string(),
        "localhost".to_string(),
        "*.localhost".to_string(),
        "*.kasuki.moe".to_string(),
    ];

    // Generate the certificate and private key
    let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();

    let private_key = cert.key_pair.serialize_pem();
    let certificate = cert.cert.pem();
    trace!("Private key: {}", private_key);
    trace!("Certificate: {}", certificate);

    let private_key_path = GRPC_KEY_PATH.clone();
    let cert_path = GRPC_CERT_PATH.clone();

    // create all the directories in the path if they don't exist except the last one
    let parent = std::path::Path::new(&private_key_path).parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    // do the same for the cert path
    let parent = std::path::Path::new(&cert_path).parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();

    std::fs::write(private_key_path, private_key).unwrap();
    std::fs::write(cert_path, certificate).unwrap();
}
