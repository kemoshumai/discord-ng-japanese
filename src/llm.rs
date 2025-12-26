use anyhow::Result;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};

#[derive(Clone, Debug)]
enum MessageRole {
    User,
    Assistant,
}

#[derive(Clone, Debug)]
struct Message {
    role: MessageRole,
    content: String,
}

#[derive(Clone, Default, Debug)]
pub struct History {
    messages: Vec<Message>,
}

impl History {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn push_as_user(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: content.to_string(),
        });
    }

    pub fn push_as_assistant(&mut self, content: &str) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: content.to_string(),
        });
    }

    pub fn push_as_system(&mut self, content: &str) {
        // llm-rs doesn't have a system message type, so we use user message instead
        self.push_as_user(content);
    }

    pub fn get_with_user(&self, content: &str) -> Self {
        let mut history = self.clone();
        history.push_as_user(content);
        history
    }

    pub fn get_with_assistant(&self, content: &str) -> Self {
        let mut history = self.clone();
        history.push_as_assistant(content);
        history
    }

    pub fn get_with_system(&self, content: &str) -> Self {
        // For llm-rs compatibility, prepend system prompt to the first user message
        let mut history = Self::new();

        if self.messages.is_empty() {
            // If no messages, just add the system prompt as a user message
            history.push_as_user(content);
        } else {
            // Find the first user message and prepend system prompt to it
            let mut system_added = false;
            for msg in &self.messages {
                match msg.role {
                    MessageRole::User if !system_added => {
                        // Prepend system prompt to the first user message
                        let merged_content = format!("{}\n\n{}", content, msg.content);
                        history.push_as_user(&merged_content);
                        system_added = true;
                    }
                    MessageRole::User => {
                        history.push_as_user(&msg.content);
                    }
                    MessageRole::Assistant => {
                        history.push_as_assistant(&msg.content);
                    }
                }
            }

            // If no user message was found, add system prompt at the beginning
            if !system_added {
                let mut result = Self::new();
                result.push_as_user(content);
                for msg in &self.messages {
                    match msg.role {
                        MessageRole::User => result.push_as_user(&msg.content),
                        MessageRole::Assistant => result.push_as_assistant(&msg.content),
                    }
                }
                return result;
            }
        }

        history
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn to_chat_messages(&self) -> Vec<ChatMessage> {
        self.messages
            .iter()
            .map(|msg| match msg.role {
                MessageRole::User => ChatMessage::user().content(&msg.content).build(),
                MessageRole::Assistant => ChatMessage::assistant().content(&msg.content).build(),
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub async fn request_mut(&mut self, model: &str) -> Result<String> {
        let response_in_text = self.request(model).await?;

        self.push_as_assistant(response_in_text.as_str());

        Ok(response_in_text)
    }

    pub async fn request(&self, model: &str) -> Result<String> {
        tracing::info!("Starting LLM request with model: {}", model);
        tracing::debug!("Message count: {}", self.messages.len());

        if self.messages.is_empty() {
            tracing::warn!("Attempting to make LLM request with empty message history");
            return Ok(String::new());
        }

        let api_key =
            std::env::var("OPENAI_API_KEY").expect("Expected OPENAI_API_KEY in environment");

        tracing::debug!("Building LLM client...");
        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(api_key)
            .model(model)
            .max_tokens(2048)
            // temperature removed - some models don't support it
            .build()
            .expect("Failed to build LLM (OpenAI)");

        let messages = self.to_chat_messages();
        tracing::debug!("Sending chat request with {} messages...", messages.len());

        // Log the first few characters of each message for debugging
        for (i, msg) in self.messages.iter().enumerate() {
            let preview = if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            };
            tracing::debug!("Message {}: {:?} - {}", i, msg.role, preview);
        }

        let response = llm.chat(&messages).await.map_err(|e| {
            tracing::error!("LLM chat request failed: {:?}", e);
            e
        })?;

        tracing::debug!("Extracting response text...");
        let response_in_text = response.text().unwrap_or_default().to_string();

        if response_in_text.is_empty() {
            tracing::warn!("LLM returned empty response!");
        } else {
            tracing::info!(
                "LLM response received, length: {} chars",
                response_in_text.len()
            );
            tracing::debug!("Response content: {}", response_in_text);
        }

        Ok(response_in_text)
    }

    pub async fn rollup(&mut self, n: u8) -> Result<()> {
        if n == 0 {
            self.clear();
            return Ok(());
        }
        let len = self.messages.len();
        let start = len - len / n as usize;
        self.messages = self.messages[start..].to_vec();
        Ok(())
    }
}
