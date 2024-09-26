use std::{collections::HashMap, env, sync::Arc};

use songbird::{shards::TwilightMap, Config, Songbird};
use tokio::sync::Mutex;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::{gateway::payload::incoming::MessageCreate, id::Id};
use vesper::prelude::Framework;

mod ng_japanese;
mod llm;
mod assistant;
mod ping;
mod slot;
mod dice;
mod voice_chat;

pub type Message = Box<MessageCreate>;
pub struct Context{
    pub history: Arc<Mutex<llm::History>>,
    pub songbird: Arc<Songbird>,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let token = env::var("DISCORD_TOKEN")?;

    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT | Intents::GUILD_VOICE_STATES,
    );

    let http = twilight_http::Client::new(token);
    let user_id = http.current_user().await?.model().await?.id;
    let http = Arc::new(http);

    let cache = InMemoryCache::builder().message_cache_size(10).build();

    let shard_hashmap = {
        let mut map = HashMap::new();
        map.insert(shard.id().number(),shard.sender());
        map
    };

    let songbird_config = Config::default().decode_mode(songbird::driver::DecodeMode::Decode);

    let songbird = Songbird::twilight(Arc::new(TwilightMap::new(shard_hashmap)), user_id);
    songbird.set_config(songbird_config);
    let songbird = Arc::new(songbird);

    let context = Arc::new(Context {
        history: Arc::new(Mutex::new(llm::History::new())),
        songbird: Arc::clone(&songbird),
    });

    let application_id = Id::new(env::var("APPLICATION_ID")?.parse()?);
    let framework = Arc::new(Framework::builder(http.clone(), application_id, context.clone())
        .command(ping::ping)
        .command(assistant::reset)
        .command(assistant::rollup)
        .command(slot::kemoshumai_slot)
        .command(dice::dice)
        .command(dice::random)
        .command(voice_chat::join)
        .command(voice_chat::leave)
        .build()
    );
    framework.register_guild_commands(Id::new(env::var("GUILD_ID")?.parse()?)).await?;

    // Process each event as they come in.
    loop{
        let item = shard.next_event().await;
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
            continue;
        };

        // Songbirdのイベントを処理
        songbird.process(&event).await;

        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&http), Arc::clone(&context), Arc::clone(&framework)));
    }
}

async fn handle_event(
    event: Event,
    http: Arc<twilight_http::Client>,
    context: Arc<Context>,
    framework: Arc<Framework<Arc<Context>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    match event {
        Event::MessageCreate(msg) => {

            ping::ping_message(&http, &context, &msg).await?;
            ng_japanese::ng_japanese(&http, &context, &msg).await?;
            assistant::assistant(&http, &context, &msg).await?;

        },
        Event::InteractionCreate(i) => {
            tokio::spawn(async move {
                let inner = i.0;
                framework.process(inner).await;
            });
        },
        _ => (),
    }

    Ok(())
}