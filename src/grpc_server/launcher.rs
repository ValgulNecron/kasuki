use std::sync::Arc;

use serenity::all::{Cache, Http, ShardManager};
use sysinfo::System;
use tokio::sync::RwLock;
use tracing::{error, trace};

use crate::config::{Config, GrpcCfg};
use crate::constant::BOT_COMMANDS;
use crate::event_handler::{BotData, RootUsage};
use crate::grpc_server::command_list::get_list_of_all_command;
use crate::grpc_server::service;
use crate::grpc_server::service::command::{get_command_server, CommandServices};
use crate::grpc_server::service::info::{get_info_server, InfoService};
use crate::grpc_server::service::shard::{get_shard_server, ShardService};

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
pub async fn grpc_server_launcher(
    shard_manager: &Arc<ShardManager>,
    command_usage: Arc<RwLock<RootUsage>>,
    cache: Arc<Cache>,
    http: Arc<Http>,
    config: Arc<Config>,
    bot_data: Arc<BotData>,
) {
    let grpc_config = config.grpc.clone();
    get_list_of_all_command();
    // Clone the Arc<ShardManager>
    let shard_manager_arc: Arc<ShardManager> = shard_manager.clone();
    // Define the address for the gRPC server
    let addr = format!("0.0.0.0:{}", grpc_config.grpc_port);
    // Create a new ShardService with the cloned Arc<ShardManager>
    let shard_service = ShardService {
        shard_manager: shard_manager_arc.clone(),
    };
    let info_service = InfoService {
        bot_info: bot_data,
        sys: Arc::new(RwLock::new(System::new_all())),
        os_info: Arc::new(os_info::get()),
        command_usage,
        shard_manager: shard_manager_arc.clone(),
        cache,
        http,
        config: config.clone(),
    };
    let bot_commands = BOT_COMMANDS.clone();
    let command_service = CommandServices {
        command_list: Arc::new(bot_commands),
    };

    // Configure the reflection service and register the file descriptor set for the shard service
    let reflection = match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(service::shard::proto::SHARD_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(service::info::proto::INFO_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(service::command::proto::COMMAND_FILE_DESCRIPTOR_SET)
        .build()
    {
        Ok(reflection) => reflection,
        Err(e) => {
            error!("Failed to build the reflection service: {}", e);
            return;
        }
    };

    let is_tls = grpc_config.use_tls;
    trace!("TLS: {}", is_tls);
    let mut builder = tonic::transport::Server::builder();
    if is_tls {
        let private_key_path = grpc_config.tls_key_path.clone();
        let cert_path = grpc_config.tls_cert_path.clone();
        generate_key(grpc_config.clone());
        // Load the server's key and certificate
        let key = tokio::fs::read(private_key_path).await.unwrap();
        let cert = tokio::fs::read(cert_path).await.unwrap();
        // Convert to a string
        let key = String::from_utf8(key).unwrap();
        let cert = String::from_utf8(cert).unwrap();
        // Build the gRPC server with TLS, add the ShardService and the reflection service, and serve the gRPC server
        let identity = tonic::transport::Identity::from_pem(cert, key);
        let tls_config = tonic::transport::ServerTlsConfig::new().identity(identity);
        builder = builder.tls_config(tls_config).unwrap()
    }
    let builder = builder
        .add_service(get_shard_server(shard_service))
        .add_service(get_info_server(info_service))
        .add_service(get_command_server(command_service))
        .add_service(reflection);

    // Serve the gRPC server
    builder.serve(addr.parse().unwrap()).await.unwrap()
}

fn generate_key(grpc_config: GrpcCfg) {
    // Specify the subject alternative names. Since we're not using a domain,
    // we'll just use "localhost" as an example.
    let subject_alt_names = vec![
        "127.0.0.1".to_string(),
        "localhost".to_string(),
        "*.localhost".to_string(),
        "*".to_string(),
    ];

    // Generate the certificate and private key
    let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();

    let private_key = cert.key_pair.serialize_pem();
    let certificate = cert.cert.pem();
    trace!("Private key: {}", private_key);
    trace!("Certificate: {}", certificate);

    let private_key_path = grpc_config.tls_key_path.clone();
    let cert_path = grpc_config.tls_cert_path.clone();

    // create all the directories in the path if they don't exist except the last one
    let parent = std::path::Path::new(&private_key_path).parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();
    // do the same for the cert path
    let parent = std::path::Path::new(&cert_path).parent().unwrap();
    std::fs::create_dir_all(parent).unwrap();

    std::fs::write(private_key_path, private_key).unwrap();
    std::fs::write(cert_path, certificate).unwrap();
}
