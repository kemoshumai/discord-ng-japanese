use crate::{Context, Message};

pub async fn ng_japanese(http: &twilight_http::Client, _ctx: &Context, msg: &Message) -> anyhow::Result<()>{

    let channel_id_ng_japanese = std::env::var("CHANNEL_ID_NG_JAPANESE").expect("Expected a channel ID in the environment");
    let channel_id_ng_japanese: u64 = channel_id_ng_japanese.parse().expect("Channel ID is not a number");

    // 日本語禁止チャンネル以外は無視
    if msg.channel_id != channel_id_ng_japanese {
        return Ok(());
    }

    if is_japanese(&msg.content) {
        if let Err(err) = http.delete_messages(msg.channel_id, &[msg.id])?.await {
            println!("Error deleting message: {err:?}");
        } else {
            println!("Deleted message: {}", msg.content);
        }
    }

    Ok(())

}


fn is_japanese(text: &str) -> bool {
    // 日本語の文字が含まれているかどうかを判定
    text.chars().any(|c| {
        let c = c as u32;
        (0x3040..=0x30FF).contains(&c) || (0x3400..=0x4DBF).contains(&c) || (0x4E00..=0x9FFF).contains(&c) || (0xF900..=0xFAFF).contains(&c) || (0xFF66..=0xFF9F).contains(&c)
    })
}