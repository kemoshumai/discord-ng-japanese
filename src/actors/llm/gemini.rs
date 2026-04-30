use std::sync::Arc;

use kameo::prelude::*;

use crate::actors::llm::{LlmContent, LlmRequest};

#[derive(Actor)]
pub struct GeminiActor {
    api_key: String,
}

impl GeminiActor {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn new_with_env() -> Self {
        let api_key = std::env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        Self { api_key }
    }
}

impl Message<LlmRequest> for GeminiActor {
    type Reply = Arc<LlmContent>;

    async fn handle(
        &mut self,
        request: LlmRequest,
        _: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        LlmContent::Assistant(format!("Received: {:?}", request)).into()
    }
}
