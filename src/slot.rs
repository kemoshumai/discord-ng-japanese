use std::sync::Arc;

use rand::Rng;
use twilight_model::http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType};
use vesper::{macros::command, prelude::{DefaultCommandResult, SlashContext}};

use crate::Context;

#[command]
#[description = "けもシューマイの亜種を運試しする"]
pub async fn kemoshumai_slot(ctx: &mut SlashContext<Arc<Context>>) -> DefaultCommandResult {

    let kemoshumai = make_kemoshumai_random();

    ctx.interaction_client.create_response(
        ctx.interaction.id,
        &ctx.interaction.token,
        &InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(InteractionResponseData {
                content: Some(kemoshumai),
                ..Default::default()
            })
        }
    ).await?;

    Ok(())
}

fn make_kemoshumai_random() -> String {
    let kemo = get_random_element_from_list(&["けも", "にく", "すし", "かも", "さけ", "にせ", "ねこ", "いぬ", "えび", "かに", "たこ", "いか", "ひも", "へび", "とら", "めか"]);
    let shumai = get_random_element_from_list(&["シューマイ", "ラーメン", "ギョウザ", "チャーハン", "ヤクザ", "マラカス", "チワワ", "ニワトリ", "ニャンコ", "ドラゴン"]);
    format!("{}{}", kemo, shumai)
}

fn get_random_element_from_list<T>(list: &[T]) -> &T{
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..list.len());
    &list[index]
}