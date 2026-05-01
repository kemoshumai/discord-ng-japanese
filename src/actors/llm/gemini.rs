use std::sync::Arc;

use kameo::prelude::*;
use llm::{builder::LLMBuilder, chat::ChatMessage, LLMProvider};

use crate::actors::llm::{LlmContent, LlmRequest};

#[derive(Actor)]
pub struct GeminiActor {
    llm_provider: Box<dyn LLMProvider>,
}

impl GeminiActor {
    pub fn new(api_key: String) -> anyhow::Result<Self> {
        let llm = LLMBuilder::new()
            .backend(llm::builder::LLMBackend::Google)
            .api_key(&api_key)
            .model("gemini-3-flash-preview")
            .build()?;
        Ok(Self { llm_provider: llm })
    }

    pub fn new_with_env() -> anyhow::Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        Self::new(api_key)
    }
}

impl Message<LlmRequest> for GeminiActor {
    type Reply = anyhow::Result<Arc<LlmContent>>;

    async fn handle(
        &mut self,
        request: LlmRequest,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let messages: Vec<ChatMessage> = request
            .contents
            .into_iter()
            .map(|content| content.into())
            .collect();

        let response = self.llm_provider.chat(&messages).await?;

        Ok(Arc::new(LlmContent::Assistant(response.to_string())))
    }
}

impl Into<ChatMessage> for LlmContent {
    fn into(self) -> ChatMessage {
        match self {
            LlmContent::User(content) => ChatMessage::user().content(content).build(),
            LlmContent::Assistant(content) => ChatMessage::assistant().content(content).build(),
        }
    }
}
