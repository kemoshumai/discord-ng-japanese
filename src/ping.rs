use crate::{Context, Message};

pub async fn ping(http: &twilight_http::Client, _ctx: &Context, msg: &Message) -> anyhow::Result<()>{

    if msg.content == "!ping" {
        if let Err(err) = http.create_message(msg.channel_id).content("Pong!")?.await {
            println!("Error sending message: {err:?}");
        } else {
            println!("Sent message: Pong!");
        }
    }

    Ok(())

}