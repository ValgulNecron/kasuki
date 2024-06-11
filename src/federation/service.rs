pub(crate) mod proto {
    // Include the protobuf definitions for the shard service
    tonic::include_proto!("federation");
    // FILE_DESCRIPTOR_SET is a constant byte array that contains the file descriptor set for the shard service
    pub(crate) const FEDERATION_FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("federation_descriptor");
}

#[derive(Debug, Clone)]
pub struct Node{
 pub    federation_name: String,
    pub federation_url: String,
    pub secondary_url: Vec<String>
}

#[derive(Debug)]
pub struct FederationService{
    pub node: Arc<RwLock<Vec<Node>>> ,
}

impl proto::federation_server::Federation for FederationService{
    
}