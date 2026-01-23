use std::sync::Arc;

use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use vesper::{
    macros::command,
    prelude::{DefaultCommandResult, SlashContext},
};

use crate::Context;

#[command]
#[description = "bet"]
pub async fn bet(
    ctx: &mut SlashContext<Arc<Context>>,
    #[description = "賭けるポストのURL"] url: String,
) -> DefaultCommandResult {
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
