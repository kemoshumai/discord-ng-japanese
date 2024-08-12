use std::env;

use serenity::all::{Context, Message};

use crate::llm::History;

pub async fn assistant(ctx: &Context, msg: &Message, history: &mut History) -> anyhow::Result<()> {

    let channel_id_ng_japanese = std::env::var("CHANNEL_ID_ASSISTANT").expect("Expected a channel ID in the environment");
    let channel_id_ng_japanese: u64 = channel_id_ng_japanese.parse().expect("Channel ID is not a number");

    // アシスタントが返事できるチャンネル以外は無視
    if msg.channel_id != channel_id_ng_japanese {
        return Ok(());
    }

    // botには返事しない
    if msg.author.bot {
        return Ok(());
    }

    history.push_as_user(&msg.content);

    let history_system = history.get_with_system(&env::var("ASSISTANT_SYSTEM").unwrap_or("かよわい女の子のような口調で返信してください。女の子の名前はミーシェです。女の子はご主人様と会話しています。".to_string()));

    let response = history_system.request(&env::var("ASSISTANT_MODEL").unwrap_or("gpt-4o".to_string())).await?;

    msg.channel_id.say(&ctx.http, &response).await?;

    history.push_as_assistant(&response);

    Ok(())
}