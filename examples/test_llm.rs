use anyhow::Result;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    println!("Testing LLM integration...\n");

    let api_key = std::env::var("OPENAI_API_KEY").expect("Expected OPENAI_API_KEY in environment");

    let model = std::env::var("VOICE_CHAT_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    println!("Using model: {}", model);

    // Test 1: Simple request
    println!("\n=== Test 1: Simple request ===");
    let messages = vec![ChatMessage::user()
        .content("Hello, please respond with 'Hello from AI'")
        .build()];

    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(api_key.clone())
        .model(&model)
        .max_tokens(512)
        .build()
        .expect("Failed to build LLM");

    match llm.chat(&messages).await {
        Ok(response) => {
            if let Some(text) = response.text() {
                println!("✓ Response: {}", text);
            } else {
                println!("✗ No text in response");
            }
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
            return Err(e.into());
        }
    }

    // Test 2: Request with system-like prompt (simulating voice_chat usage)
    println!("\n=== Test 2: Request with system prompt (merged into first user message) ===");
    let system_prompt = "You are a helpful assistant. Please respond in a friendly manner.";
    let user_message = "Hello! What is your name?";
    let merged_message = format!("{}\n\n{}", system_prompt, user_message);

    println!("Merged message: {}", merged_message);

    let messages = vec![ChatMessage::user().content(&merged_message).build()];

    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(api_key.clone())
        .model(&model)
        .max_tokens(512)
        .build()
        .expect("Failed to build LLM");

    match llm.chat(&messages).await {
        Ok(response) => {
            if let Some(text) = response.text() {
                println!("✓ Response: {}", text);
                if text.is_empty() {
                    println!("⚠ WARNING: Response is empty!");
                }
            } else {
                println!("✗ No text in response");
            }
            if let Some(usage) = response.usage() {
                println!(
                    "  Tokens used: {} prompt + {} completion",
                    usage.prompt_tokens, usage.completion_tokens
                );
            }
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
            return Err(e.into());
        }
    }

    // Test 3: Multiple messages
    println!("\n=== Test 3: Multiple messages in conversation ===");
    let messages = vec![
        ChatMessage::user().content("What is 2+2?").build(),
        ChatMessage::assistant().content("2+2 equals 4.").build(),
        ChatMessage::user().content("What about 3+3?").build(),
    ];

    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key(api_key)
        .model(&model)
        .max_tokens(512)
        .build()
        .expect("Failed to build LLM");

    match llm.chat(&messages).await {
        Ok(response) => {
            if let Some(text) = response.text() {
                println!("✓ Response: {}", text);
            } else {
                println!("✗ No text in response");
            }
        }
        Err(e) => {
            println!("✗ Error: {:?}", e);
            return Err(e.into());
        }
    }

    println!("\n✓ All tests completed successfully!");
    Ok(())
}
