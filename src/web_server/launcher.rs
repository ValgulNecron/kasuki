use tonic::{Request, Response, Status};

use proto::shard_server::Shard;

mod proto {
    tonic::include_proto!("shard");
}

#[derive(Debug, Default)]
struct ShardService {}

#[tonic::async_trait]
impl Shard for ShardService {
    async fn shard_count(
        &self,
        request: Request<proto::ShardCountRequest>,
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
        request: Request<proto::ShardInfoRequest>,
    ) -> Result<Response<proto::ShardInfoResponse>, Status> {
        let reply = proto::ShardInfoResponse {
            shard_id: 1,
            latency: "shard1".to_string(),
            stage: "2".to_string(),
        };
        Ok(Response::new(reply))
    }
}

pub async fn web_server_launcher() {}
