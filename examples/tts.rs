#[tokio::main]
async fn main() -> anyhow::Result<()>{

    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    tracing::info!("TTS Started");

    let _speech = text_to_speech("こんにちは").await?;

    tracing::info!("TTS Finished");

    Ok(())
}

async fn text_to_speech(text: &str) -> anyhow::Result<Vec<u8>> {

    let client = reqwest::Client::new();

    let url = std::env::var("COEIRO_API_URL")?;

    let response = client.post(url)
        .json(&serde_json::json!({
            "speakerUuid": "292ea286-3d5f-f1cc-157c-66462a6a9d08",
            "styleId": 42,
            "text": text,
            "speedScale": 1.2,
            "volumeScale": 1.0,
            "prosodyDetail": [],
            "pitchScale": 0.0,
            "intonationScale": 1.2,
            "prePhonemeLength": 0.1,
            "postPhonemeLength": 0.5,
            "outputSamplingRate": 24000,
        }))
        .header("Content-Type", "application/json")
        .send()
        .await?
        .bytes()
        .await?;

    Ok(response.to_vec())
}