use std::sync::Arc;

use serenity::all::{ShardId, ShardManager};
use tonic::{Request, Response, Status};

use proto::shard_server::Shard;

use crate::constant::WEB_SERVER_PORT;
use crate::web_server::launcher::proto::shard_server::ShardServer;

mod proto {
    tonic::include_proto!("shard");
}

#[derive(Debug)]
struct ShardService {
    pub shard_manager: Arc<ShardManager>,
}

#[tonic::async_trait]
impl Shard for ShardService {
    async fn shard_count(
        &self,
        _request: Request<proto::ShardCountRequest>,
    ) -> Result<Response<proto::ShardCountResponse>, Status> {
        let shard_manager = self.shard_manager.clone();
        let mut shard_ids = Vec::new();
        for (shard_id, _) in shard_manager.runners.lock().await.iter() {
            shard_ids.push(shard_id.0 as i32);
        }
        let reply = proto::ShardCountResponse {
            count: shard_ids.len() as i32,
            shard_ids,
        };
        Ok(Response::new(reply))
    }

    async fn shard_info(
        &self,
        request: Request<proto::ShardInfoRequest>,
    ) -> Result<Response<proto::ShardInfoResponse>, Status> {
        let data = request.into_inner();
        let id = data.shard_id;
        let shard_manager = self.shard_manager.clone();
        let runners = shard_manager.runners.lock().await;
        if !runners.contains_key(&ShardId(id as u32)) {
            return Err(Status::not_found("Shard not found"));
        }
        let shard = runners.get(&ShardId(id as u32)).unwrap();
        let reply = proto::ShardInfoResponse {
            shard_id: id,
            latency: shard.latency.unwrap_or_default().as_millis().to_string(),
            stage: shard.stage.to_string(),
        };
        Ok(Response::new(reply))
    }
}

pub async fn web_server_launcher(shard_manager: &Arc<ShardManager>) {
    let shard_manager_arc: Arc<ShardManager> = shard_manager.clone();

    let addr = format!("0.0.0.0:{}", *WEB_SERVER_PORT);
    let shard_service = ShardService {
        shard_manager: shard_manager_arc,
    };

    tonic::transport::Server::builder()
        .add_service(ShardServer::new(shard_service))
        .serve(addr.parse().unwrap())
        .await
        .unwrap();
}
