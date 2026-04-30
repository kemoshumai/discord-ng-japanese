use std::sync::Arc;

use anyhow::Ok;
use songbird::Songbird;
use tokio::sync::Mutex;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::discord_loop::run_discord_event_loop_with_dotenv;

mod actors;
mod assistant;
mod discord_loop;
mod llm;
mod voice_chat;

pub type Message = Box<MessageCreate>;
pub struct Context {
    pub history: Arc<Mutex<llm::History>>,
    pub songbird: Arc<Songbird>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    // Discordのイベントループを別のタスクで実行
    tokio::spawn(async {
        if let Err(err) = run_discord_event_loop_with_dotenv().await {
            println!("Error running Discord event loop: {err:?}");
        }
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
