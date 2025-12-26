use std::sync::Arc;

use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::Context;

#[command]
#[description = "ミーシェにNSFWロールをおねだりする"]
pub async fn role_nsfw(ctx: &mut SlashContext<Arc<Context>>,
    #[description = "ミーシェに渡すお金（円）"] n: u32
) -> DefaultCommandResult {

    if n < 1000000 {
        ctx.interaction_client.create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!("{}円とか、そんなはした金でNSFWロールが貰えると思ってるの？ ざーこ", n).to_string()),
                    ..Default::default()
                })
            }
        ).await?;
    } else {
        ctx.interaction_client.create_response(
            ctx.interaction.id,
            &ctx.interaction.token,
            &InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(format!("{}円？！しょ、しょうがないな～... ミーシェがNSFWロール、あげるねっ", n).to_string()),
                    ..Default::default()
                })
            }
        ).await?;

        // ユーザーにNSFWロールを付与する処理
        let guild_id = ctx.interaction.guild_id.expect("Guild ID is required");
        let user_id = ctx.interaction.author_id().expect("User ID is required");
        let role_id = std::env::var("ROLE_NSFW")?.parse()?;
        ctx.http_client().add_guild_member_role(guild_id, user_id, role_id).await.expect("Failed to add NSFW role");
    }

    Ok(())
}