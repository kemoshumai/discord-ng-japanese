use discord_ng_japanese::actors::llm::{gemini::GeminiActor, LlmContent, LlmRequest};
use kameo::actor::Spawn;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    // LlmRouterActorをスポーンしてみる
    let actor = GeminiActor::spawn(GeminiActor::new_with_env()?);

    // メッセージを渡す
    let response = actor.ask(LlmRequest {
        contents: vec![LlmContent::User("もしもし".to_string())],
    });

    println!("{:?}", response.await);

    Ok(())
}
