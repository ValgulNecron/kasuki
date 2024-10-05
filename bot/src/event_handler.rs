use std::sync::Arc;
use serenity::all::{Context, Ready};
use serenity::prelude::EventHandler;

pub struct Handler {
    pub bot_data: Arc<String>
}

impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        todo!()
    }
}