use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::{Context, Message};

pub async fn ping_message(http: &twilight_http::Client, _ctx: &Context, msg: &Message) -> anyhow::Result<()>{

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
pub async fn ping(ctx: &mut SlashContext<()>) -> DefaultCommandResult {
    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(String::from("pong!")),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
