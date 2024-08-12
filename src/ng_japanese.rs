use std::env;

use serenity::all::{Context, Message};

use crate::llm::chat_once;

pub async fn ng_japanese(ctx: Context, msg: Message) {

    let channel_id_ng_japanese = std::env::var("CHANNEL_ID_NG_JAPANESE").expect("Expected a channel ID in the environment");
    let channel_id_ng_japanese: u64 = channel_id_ng_japanese.parse().expect("Channel ID is not a number");

    // 日本語禁止チャンネル以外は無視
    if msg.channel_id != channel_id_ng_japanese {
        return;
    }

    // 日本語を含むメッセージを削除

    let is_japanese = chat_once(&env::var("NG_JAPANESE_MODEL").unwrap_or("gpt-4o-mini".to_string()), format!("この文章は日本語で書かれていますか？なお、アルファベットで書かれた日本語の文章などはYesと答え、日本語を含む英語の文章はNoと答えてください。「Yes」か「No」かで答えてください。\n\n{}", msg.content).as_str()).await.unwrap() == "Yes";

    if is_japanese {
        if let Err(err) = msg.delete(&ctx.http).await {
            println!("Error deleting message: {err:?}");
        } else {
            println!("Deleted message: {}", msg.content);
        }
    }

}