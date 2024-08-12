use serenity::all::{Context, Message};

pub async fn ng_japanese(ctx: Context, msg: Message) {

    let channel_id_ng_japanese = std::env::var("CHANNEL_ID_NG_JAPANESE").expect("Expected a channel ID in the environment");
    let channel_id_ng_japanese: u64 = channel_id_ng_japanese.parse().expect("Channel ID is not a number");

    // 日本語禁止チャンネル以外は無視
    if msg.channel_id != channel_id_ng_japanese {
        return;
    }

    // 日本語を含むメッセージを削除
    if msg.content.contains(|c: char| ('\u{3040}'..='\u{30ff}').contains(&c)) {
        if let Err(err) = msg.delete(&ctx.http).await {
            println!("Error deleting message: {err:?}");
        }
    }

}