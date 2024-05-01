use std::sync::Arc;

use serenity::all::{ShardId, ShardManager};
use tonic::{Request, Response, Status};

use proto::shard_server::Shard;

use crate::constant::GRPC_SERVER_PORT;
use crate::grpc_server::launcher::proto::shard_server::ShardServer;

// Proto module contains the protobuf definitions for the shard service
mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("shard");

    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("shard_descriptor");
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
        // Return the ShardCountResponse
        Ok(Response::new(reply))
    }

    // shard_info is an async function that returns the information of a specific shard
    // It takes a Request<proto::ShardInfoRequest> as a parameter and returns a Result<Response<proto::ShardInfoResponse>, Status>
    async fn shard_info(
        &self,
        request: Request<proto::ShardInfoRequest>,
    ) -> Result<Response<proto::ShardInfoResponse>, Status> {
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
        // Return the ShardInfoResponse
        Ok(Response::new(reply))
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

    // Configure the reflection service and register the file descriptor set for the shard service
    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();

    // Build the gRPC server, add the ShardService and the reflection service, and serve the gRPC server
    tonic::transport::Server::builder()
        .add_service(ShardServer::new(shard_service))
        .add_service(reflection)
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
