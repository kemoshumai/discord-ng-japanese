use std::{num::NonZeroU64, sync::Arc};

use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

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

    ctx.data.songbird.join(guild_id, channel_id).await?;

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