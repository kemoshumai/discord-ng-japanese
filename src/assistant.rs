use std::{env, sync::Arc};

use twilight_model::{http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType}, id::Id};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::{Context, Message};

pub async fn assistant(http: &twilight_http::Client, ctx: &Context, msg: &Message) -> anyhow::Result<()> {

    let mut history = ctx.history.lock().await;

    let channel_id = std::env::var("CHANNEL_ID_ASSISTANT").expect("Expected a channel ID in the environment");
    let channel_id: u64 = channel_id.parse().expect("Channel ID is not a number");

    // アシスタントが返事できるチャンネル以外は無視
    if msg.channel_id != channel_id {
        return Ok(());
    }

    // botには返事しない
    if msg.author.bot {
        return Ok(());
    }

    history.push_as_user(&msg.content);

    let history_system = history.get_with_system(&env::var("ASSISTANT_SYSTEM").unwrap_or("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と会話しています。".to_string()));

    let response = history_system.request(&env::var("ASSISTANT_MODEL").unwrap_or("gpt-4o".to_string())).await?;

    let _ = http.create_message(Id::new(channel_id)).content(&response)?.await;

    history.push_as_assistant(&response);

    Ok(())
}


#[command]
#[description = "アシスタントとの会話をリセットする"]
pub async fn reset(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some("（会話がリセットされました）".to_string()),
                ..Default::default()
            })
        }
    ).await?;

    let mut history = ctx.data.history.lock().await;
    history.clear();

    Ok(())
}
