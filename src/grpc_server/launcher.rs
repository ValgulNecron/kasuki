use std::sync::{Arc, RwLock};

use serenity::all::{CurrentApplicationInfo, ShardId, ShardManager};
use sysinfo::System;
use tonic::{Request, Response, Status};
use tracing::trace;

use crate::constant::{
    ACTIVITY_NAME, APP_VERSION, BOT_COMMANDS, BOT_INFO, GRPC_CERT_PATH, GRPC_KEY_PATH,
    GRPC_SERVER_PORT, GRPC_USE_TLS,
};
use crate::grpc_server::command_list::{
    get_list_of_all_command, Arg, Command, CommandItem, SubCommand, SubCommandGroup,
};
use crate::grpc_server::service;
use crate::grpc_server::service::command::{CommandServices, get_command_server};
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
pub async fn grpc_server_launcher(shard_manager: &Arc<ShardManager>) {
    get_list_of_all_command();
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
    let command_service = unsafe {
        CommandServices {
            command_list: Arc::new(BOT_COMMANDS.clone()),
        }
    };

    // Configure the reflection service and register the file descriptor set for the shard service
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(service::shard::proto::SHARD_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(service::info::proto::INFO_FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(service::command::proto::COMMAND_FILE_DESCRIPTOR_SET)
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
            .add_service(get_shard_server(shard_service))
            .add_service(get_info_server(info_service))
            .add_service(get_command_server(command_service))
            .add_service(reflection)
            .serve(addr.parse().unwrap())
            .await
            .unwrap();
    } else {
        // Build the gRPC server, add the ShardService and the reflection service, and serve the gRPC server
        tonic::transport::Server::builder()
            .add_service(get_shard_server(shard_service))
            .add_service(get_info_server(info_service))
            .add_service(get_command_server(command_service))
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
