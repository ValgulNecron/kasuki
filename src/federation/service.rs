use crate::federation::service::proto::federation_connection_service_server::FederationConnectionService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::Request;

pub(crate) mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("federation");
    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const FEDERATION_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("federation_descriptor");
}

#[derive(Debug, Clone)]
pub struct Node {
    pub federation_name: String,
    pub federation_url: String,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token: String,
    pub generated_at: String,
}

#[derive(Debug)]
pub struct FederationServices {
    pub node: Arc<RwLock<HashMap<String, Node>>>,
    pub token: Arc<RwLock<HashMap<String, Token>>>,
    pub name: String,
    pub url: String,
}

use crate::federation::service::proto::ConnectResponse;
use crate::federation::service::proto::{ConnectRequest, RenewTokenRequest, RenewTokenResponse};
use tonic::Response;
use tonic::Status;

impl FederationServices {
    pub async fn add_token(&self, node_name: String, token: String) {
        let tokens = self.token.clone();
        let mut guard = tokens.write().await;
        guard.insert(
            node_name.clone(),
            Token {
                token: token.clone(),
                generated_at: chrono::Utc::now().to_string(),
            },
        );
    }

    pub async fn remove_token(&self, node_name: String, token: String) {
        if self.verify_token(node_name.clone(), token.clone()).await {
            return;
        }
        let tokens = self.token.clone();
        let mut guard = tokens.write().await;
        guard.remove(&node_name);
    }

    pub async fn verify_token(&self, node_name: String, token: String) -> bool {
        self.remove_expired_token().await;
        let tokens = self.token.clone();
        let guard = tokens.read().await;
        match guard.get(&node_name) {
            Some(t) => {
                if t.token == token {
                    return true;
                }
            }
            None => {
                return false;
            }
        }
        false
    }

    pub async fn renew_token(&self, node_name: String, token: String) {
        let tokens = self.token.clone();
        let mut guard = tokens.write().await;
        guard.insert(
            node_name.clone(),
            Token {
                token: token.clone(),
                generated_at: chrono::Utc::now().to_string(),
            },
        );
    }

    pub async fn remove_expired_token(&self) {
        let tokens = self.token.clone();
        let r_guard = tokens.read().await;
        let mut guard = tokens.write().await;
        for (k, v) in r_guard.iter() {
            let generated_at = chrono::DateTime::parse_from_rfc3339(&v.generated_at).unwrap();
            let now = chrono::Utc::now();
            let duration = now.signed_duration_since(generated_at);
            if duration.num_hours() > 1 {
                guard.remove(k);
            }
        }
    }

    pub async fn add_node(&self, node_name: String, node: Node) {
        let mut nodes = self.node.write().await;
        nodes.insert(node_name, node);
    }

    pub async fn remove_node(&self, node_name: String) {
        let mut node = self.node.write().await;
        node.remove(&node_name);
    }
}

use crate::federation::service::proto::DisconnectRequest;
use crate::federation::service::proto::DisconnectResponse;

#[tonic::async_trait]
impl FederationConnectionService for FederationServices {
    async fn connect_federation(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<ConnectResponse>, Status> {
        let request = request.into_inner();
        let node_name = request.self_name;
        let secondary_url = request.self_url;
        let generated_token = uuid::Uuid::new_v4().to_string();
        self.add_token(node_name.clone(), generated_token.clone())
            .await;
        self.add_node(
            node_name.clone(),
            Node {
                federation_name: node_name.clone(),
                federation_url: secondary_url.clone(),
            },
        )
        .await;

        Ok(Response::new(ConnectResponse {
            federation_name: self.name.clone(),
            federation_url: self.url.clone(),
            token: generated_token,
        }))
    }

    async fn disconnect_federation(
        &self,
        request: Request<DisconnectRequest>,
    ) -> Result<Response<DisconnectResponse>, Status> {
        let request = request.into_inner();
        let node_name = request.self_name;
        let token = request.token;
        if !self.verify_token(node_name.clone(), token.clone()).await {
            Err(Status::unauthenticated("Invalid token"))?
        }

        self.remove_node(node_name.clone()).await;
        self.remove_token(node_name.clone(), token.clone()).await;
        return Ok(Response::new(DisconnectResponse { success: true }));
    }

    async fn renew_token_federation(
        &self,
        request: Request<RenewTokenRequest>,
    ) -> Result<Response<RenewTokenResponse>, Status> {
        let request = request.into_inner();
        let node_name = request.self_name;
        let token = request.token;
        if !self.verify_token(node_name.clone(), token.clone()).await {
            Err(Status::unauthenticated("Invalid token"))?
        }
        let generated_token = uuid::Uuid::new_v4().to_string();
        self.renew_token(node_name.clone(), generated_token.clone())
            .await;
        Ok(Response::new(RenewTokenResponse {
            token: generated_token,
        }))
    }
}
