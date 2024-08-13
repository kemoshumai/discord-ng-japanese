use std::sync::Arc;

use rand::Rng;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::Context;


#[command]
#[description = "さいころを振る"]
pub async fn dice(ctx: &mut SlashContext<Arc<Context>>,
    #[description = "さいころの目の数の個数"] dice_1d: u8
) -> DefaultCommandResult {

    let n = rand::thread_rng().gen_range(1..=dice_1d);

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(format!("{}です！", n).to_string()),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}
