use songbird::{shards::TwilightMap, Config, Songbird};
use std::{collections::HashMap, sync::Arc};
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_model::id::Id;
use vesper::prelude::Framework;

mod dice;
mod ng_japanese;
mod ping;
mod role;
mod slot;

pub struct FrameworkContext {
    songbird: Arc<Songbird>,
}

pub async fn run_discord_event_loop_with_dotenv() -> anyhow::Result<()> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let application_id = std::env::var("APPLICATION_ID")?;
    let guild_id = std::env::var("GUILD_ID")?;
    run_discord_event_loop(&token, &application_id, &guild_id).await
}

pub async fn run_discord_event_loop(
    token: &str,
    application_id: &str,
    guild_id: &str,
) -> anyhow::Result<()> {
    let mut shard = Shard::new(
        ShardId::ONE,
        token.to_string(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT | Intents::GUILD_VOICE_STATES,
    );

    let http = twilight_http::Client::new(token.to_string());
    let user_id = http.current_user().await?.model().await?.id;
    let http = Arc::new(http);

    let cache = InMemoryCache::builder().message_cache_size(10).build();

    let shard_hashmap = {
        let mut map = HashMap::new();
        map.insert(shard.id().number(), shard.sender());
        map
    };

    let songbird_config = Config::default().decode_mode(songbird::driver::DecodeMode::Decode);

    let songbird = Songbird::twilight(Arc::new(TwilightMap::new(shard_hashmap)), user_id);
    songbird.set_config(songbird_config);
    let songbird = Arc::new(songbird);

    let context = Arc::new(FrameworkContext {
        songbird: Arc::clone(&songbird),
    });

    let application_id = Id::new(application_id.parse()?);
    let framework = Arc::new(
        Framework::builder(Arc::clone(&http), application_id, context.clone())
            // ? アシスタント関係無いDiscordの機能5つ
            .command(crate::discord_loop::ping::ping)
            .command(crate::discord_loop::dice::dice)
            .command(crate::discord_loop::dice::random)
            .command(crate::discord_loop::slot::kemoshumai_slot)
            .command(crate::discord_loop::role::role_nsfw)
            // ? 以下はアシスタントの状態に介入するメンテナンスコマンド
            // .command(crate::assistant::reset)
            // .command(crate::assistant::rollup)
            // .command(crate::voice_chat::join)
            // .command(crate::voice_chat::leave)
            .build(),
    );
    framework
        .register_guild_commands(Id::new(guild_id.parse()?))
        .await?;

    loop {
        let item = shard.next_event().await;
        let Ok(event) = item else {
            tracing::warn!(source = ?item.unwrap_err(), "error receiving event");
            continue;
        };

        // Songbirdのイベントを処理
        songbird.process(&event).await;

        // Update the cache with the event.
        cache.update(&event);

        tokio::spawn(handle_event(
            event,
            Arc::clone(&http),
            Arc::clone(&framework),
        ));
    }
}

async fn handle_event(
    event: Event,
    http: Arc<twilight_http::Client>,
    framework: Arc<Framework<Arc<FrameworkContext>>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            // ? アシスタント関係無いDiscordの機能
            crate::discord_loop::ping::ping_message(&http, &framework, &msg).await?;
            crate::discord_loop::ng_japanese::ng_japanese(&http, &framework, &msg).await?;
            // ? 以下はアシスタントの状態に介入するイベント
            // crate::assistant::assistant(&http, &framework, &msg).await?;
        }
        Event::InteractionCreate(i) => {
            tokio::spawn(async move {
                let inner = i.0;
                framework.process(inner).await;
            });
        }
        _ => (),
    }

    Ok(())
}
