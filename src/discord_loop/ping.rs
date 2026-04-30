use std::sync::Arc;

use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use vesper::{
    macros::command,
    prelude::{DefaultCommandResult, Framework, SlashContext},
};

use crate::{discord_loop::FrameworkContext, Message};

pub async fn ping_message(
    http: &twilight_http::Client,
    _framework: &Arc<Framework<Arc<FrameworkContext>>>,
    msg: &Message,
) -> anyhow::Result<()> {
    if msg.content == "!ping" {
        if let Err(err) = http.create_message(msg.channel_id).content("Pong!")?.await {
            println!("Error sending message: {err:?}");
        } else {
            println!("Sent message: Pong!");
        }
    }

    Ok(())
}

#[command]
#[description = "ping"]
pub async fn ping(ctx: &mut SlashContext<Arc<FrameworkContext>>) -> DefaultCommandResult {
    ctx.interaction_client
        .create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(String::from("pong!")),
                    ..Default::default()
                }),
            },
        )
        .await?;

    Ok(())
}
