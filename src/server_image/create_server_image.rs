use std::time::Duration;
use tokio::time::sleep;

pub async fn create_sever_image() {

}

async fn get_server_list() {

}

async fn server_image_dispatch() {
 loop {
    sleep(Duration::from_secs(24*60*60)).await;
    let server_list = get_server_list().await;
    for server in server_list {
        create_sever_image().await;
    }
 }
}