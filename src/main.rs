use discord_ng_japanese::discord_loop::run_discord_event_loop_with_dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    // Discordのイベントループを別のタスクで実行
    tokio::spawn(async {
        if let Err(err) = run_discord_event_loop_with_dotenv().await {
            println!("Error running Discord event loop: {err:?}");
        }
    });

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
