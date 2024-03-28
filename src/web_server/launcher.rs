use std::sync::Arc;

use serenity::all::ShardManager;
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
        let mut shard_id = Vec::new();
        shard_id.insert(0, 1);
        let reply = proto::ShardCountResponse {
            count: 1,
            shard_ids: shard_id,
        };
        Ok(Response::new(reply))
    }

    async fn shard_info(
        &self,
        _request: Request<proto::ShardInfoRequest>,
    ) -> Result<Response<proto::ShardInfoResponse>, Status> {
        let reply = proto::ShardInfoResponse {
            shard_id: 1,
            latency: "shard1".to_string(),
            stage: "2".to_string(),
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
