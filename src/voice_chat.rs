use std::{num::NonZeroU64, sync::Arc};

use songbird::{CoreEvent, EventContext, EventHandler};
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{async_trait, DefaultCommandResult, SlashContext}};

use crate::Context;

#[command]
#[description = "ボイスチャンネルに招待する"]
pub async fn join(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {

    let guild_id = ctx.interaction.guild_id.unwrap();
    let channel_id: NonZeroU64 = std::env::var("VOICE_CHANNEL_ID").expect("Expected a voice channel ID in the environment").parse().expect("Voice channel ID is not a number");
    
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("おじゃまします！".to_string()),
                ..Default::default()
            })
        }
    ).await?;

    let songbird = ctx.data.songbird.clone();

    tokio::spawn(async move {
        let call = {
            match songbird.join(guild_id, channel_id).await {
                Ok(call) => call,
                Err(why) => {
                    println!("Failed to join a channel: {:?}", why);
                    songbird.get(guild_id).unwrap()
                }
            }
        };
        let mut handler = call.lock().await;
    
        let receiver = Receiver::new();

        handler.add_global_event(CoreEvent::SpeakingStateUpdate.into(), receiver.clone());
        handler.add_global_event(CoreEvent::VoiceTick.into(), receiver.clone());
    });

    Ok(())
}

#[command]
#[description = "ボイスチャンネルから追い出す"]
pub async fn leave(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {

    let guild_id = ctx.interaction.guild_id.unwrap();
    ctx.data.songbird.leave(guild_id).await?;

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("おじゃましました！".to_string()),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

#[derive(Clone)]
struct Receiver{
    inner: Arc<ReceiverContext>
}

impl Receiver {
    fn new() -> Self {
        Self {
            inner: Arc::new(ReceiverContext::new())
        }
    }
}

#[async_trait]
impl EventHandler for Receiver {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<songbird::Event>{

        let _receiver_context = self.inner.clone();

        match ctx {
            EventContext::SpeakingStateUpdate(speaking) => {
                println!("{:?}", speaking);
            },
            EventContext::VoiceTick(voice_tick) => {
                println!("{:?}", voice_tick);
            }
            _ => {}
        }

        None
    }
}


struct ReceiverContext;

impl ReceiverContext {
    fn new() -> Self {
        Self
    }
}