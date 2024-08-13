use std::{env, sync::Arc};

use twilight_model::{http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType}, id::Id};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::{llm, Context, Message};

pub async fn assistant(http: &twilight_http::Client, ctx: &Context, msg: &Message) -> anyhow::Result<()> {

    let mut history = ctx.history.lock().await;

    let channel_id = std::env::var("CHANNEL_ID_ASSISTANT").expect("Expected a channel ID in the environment");
    let channel_id: u64 = channel_id.parse().expect("Channel ID is not a number");

    let bot_role_id = env::var("BOT_ROLE_ID").expect("Expected a bot user ID in the environment");

    // アシスタントが返事できるチャンネル以外の場合
    if msg.channel_id != channel_id {

        // メンションされていたらメンションへの返事をする
        if msg.mention_roles.iter().any(|mention| *mention == Id::new(bot_role_id.parse().expect("Bot role ID is not a number"))) {
            assistant_reply_to_mentioned_post(http, ctx, msg).await?;
        }

        return Ok(());
    }

    // botには返事しない
    if msg.author.bot {
        return Ok(());
    }

    // 「.」から始まるメッセージは無視
    if msg.content.starts_with('.') {
        return Ok(());
    }

    http.create_typing_trigger(msg.channel_id).await?;

    history.push_as_user(&msg.content);

    let history_system = history.get_with_system(&env::var("ASSISTANT_SYSTEM").unwrap_or("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と会話しています。".to_string()));

    let response = history_system.request(&env::var("ASSISTANT_MODEL").unwrap_or("gpt-4o".to_string())).await?;

    let _ = http.create_message(Id::new(channel_id)).content(&response)?.await;

    history.push_as_assistant(&response);

    Ok(())
}

pub async fn assistant_reply_to_mentioned_post(http: &twilight_http::Client, _ctx: &Context, msg: &Message) -> anyhow::Result<()> {

    let res = http.channel_messages(msg.channel_id).await?.models().await?;

    // 直近10件のメッセージのみ残し、順番を古い→新しいにする
    let res = res.iter().take(10).rev().collect::<Vec<_>>();

    let mut history = llm::History::new();

    for msg in res.iter() {
        history.push_as_user(&msg.content);
    }

    history.push_as_system(&env::var("ASSISTANT_SYSTEM").unwrap_or("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と会話しています。".to_string()));

    http.create_typing_trigger(msg.channel_id).await?;

    let res = history.request(&env::var("ASSISTANT_MODEL").unwrap_or("gpt-4o".to_string())).await?;

    let _ = http.create_message(msg.channel_id).reply(msg.id).content(&res)?.await;

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

#[command]
#[description = "新しい会話のn割を残して会話をリセットする"]
pub async fn rollup(ctx: &mut SlashContext<Arc<Context>>,
    #[description = "何割を残すか"] n: u8
) -> DefaultCommandResult {

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("（{}割がリセットされました）", n).to_string()),
                ..Default::default()
            })
        }
    ).await?;

    let mut history = ctx.data.history.lock().await;
    history.rollup(n).await?;

    Ok(())
}
