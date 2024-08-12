use std::env;
use std::sync::Arc;

use llm::History;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

mod ng_japanese;
mod llm;
mod assistant;

struct Handler;

impl TypeMapKey for History {
    type Value = Arc<Mutex<History>>;
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        ng_japanese::ng_japanese(&ctx, &msg).await;

        {
            let history = {
                let data = ctx.data.read().await;
                data.get::<llm::History>().unwrap().clone()
            };
            let mut history = history.lock().await;
            let _ = assistant::assistant(&ctx, &msg, &mut history).await;
        }
    }
}

#[tokio::main]
async fn main() {

    dotenv::dotenv().expect("Failed to read .env file");

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<llm::History>(Arc::new(Mutex::new(History::new())));
    }

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}