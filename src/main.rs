use std::{env, sync::Arc};

use tokio::sync::Mutex;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::{gateway::payload::incoming::MessageCreate, id::Id};
use vesper::prelude::Framework;

mod ng_japanese;
mod llm;
mod assistant;
mod ping;

pub type Message = Box<MessageCreate>;
pub struct Context{
    pub history: Arc<Mutex<llm::History>>
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenv::dotenv().ok();

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
        history: Arc::new(Mutex::new(llm::History::new())),
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

        tokio::spawn(handle_event(event, Arc::clone(&http), Arc::clone(&context)));
    }
}

async fn handle_event(
    event: Event,
    http: Arc<twilight_http::Client>,
    context: Arc<Context>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let application_id = Id::new(env::var("APPLICATION_ID")?.parse()?);
    let framework = Arc::new(Framework::builder(http.clone(), application_id, context.clone())
        .command(ping::ping)
        .command(assistant::reset)
        .command(assistant::rollup)
        .build()
    );

    framework.register_guild_commands(Id::new(env::var("GUILD_ID")?.parse()?)).await?;

    match event {
        Event::MessageCreate(msg) => {

            ping::ping_message(&http, &context, &msg).await?;
            ng_japanese::ng_japanese(&http, &context, &msg).await?;
            assistant::assistant(&http, &context, &msg).await?;

        },
        Event::InteractionCreate(i) => {
            let clone = Arc::clone(&framework);
            tokio::spawn(async move {
                let inner = i.0;
                clone.process(inner).await;
            });
        },
        _ => (),
    }

    Ok(())
}