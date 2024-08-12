use anyhow::Result;
use async_openai::types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};

pub async fn chat_once(model: &str, prompt: &str) -> Result<String> {
    let client = async_openai::Client::new();

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(vec![
            ChatCompletionRequestUserMessageArgs::default().content(prompt).build()?.into()
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    let response_in_text = response.choices.first().unwrap().message.content.clone().unwrap_or_default();

    println!("response_in_text: {}", response_in_text);

    Ok(response_in_text)
}