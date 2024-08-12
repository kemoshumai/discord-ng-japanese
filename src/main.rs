use std::{env, sync::{Arc, Mutex}};

use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::gateway::payload::incoming::MessageCreate;

mod ng_japanese;
mod llm;
mod assistant;

pub type Message = Box<MessageCreate>;
pub struct Context{
    pub histoy: Arc<Mutex<llm::History>>
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")?;

    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    );

    let http = Arc::new(twilight_http::Client::new(token));

    let cache = InMemoryCache::builder().message_cache_size(10).build();

    let context = Arc::new(Context {
        histoy: Arc::new(Mutex::new(llm::History::new())),
    });

    // Process each event as they come in.
    loop{
        let item = shard.next_event().await;
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
            continue;
        };

        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&http), context.clone()));
    }
}

async fn handle_event(
    event: Event,
    http: Arc<twilight_http::Client>,
    context: Arc<Context>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {

            ng_japanese::ng_japanese(&http, &context, &msg).await;

        }
        _ => {}
    }

    Ok(())
}