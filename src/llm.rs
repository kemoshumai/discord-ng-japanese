use anyhow::Result;
use async_openai::types::{ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};

#[derive(Clone, Default, Debug)]
pub struct History(Vec<ChatCompletionRequestMessage>);

impl History {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, message: ChatCompletionRequestMessage) {
        self.0.push(message);
    }

    pub fn push_as_user(&mut self, content: &str) {
        self.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into()
        );
    }

    pub fn push_as_assistant(&mut self, content: &str) {
        self.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into()
        );
    }

    pub fn push_as_system(&mut self, content: &str) {
        self.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(content)
                .build()
                .unwrap()
                .into()
        );
    }

    pub fn get_with_user(&self, content: &str) -> Self{
        let mut history = self.clone();
        history.push_as_user(content);
        history
    }

    pub fn get_with_assistant(&self, content: &str) -> Self{
        let mut history = self.clone();
        history.push_as_assistant(content);
        history
    }

    pub fn get_with_system(&self, content: &str) -> Self{
        let mut history = self.clone();
        history.push_as_system(content);
        history
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn get_messages(&self) -> Vec<ChatCompletionRequestMessage> {
        self.0.clone()
    }

    pub async fn request_mut(&mut self, model: &str) -> Result<String> {
        let response_in_text = self.request(model).await?;
    
        self.push_as_assistant(response_in_text.as_str());
    
        Ok(response_in_text)
    }
    
    pub async fn request(&self, model: &str) -> Result<String> {
        let client = async_openai::Client::new();

        let request = CreateChatCompletionRequestArgs::default()
            .model(model)
            .messages(self.get_messages())
            .build()?;
    
        let response = client.chat().create(request).await?;
    
        let response_in_text = response.choices.first().unwrap().message.content.clone().unwrap_or_default();
    
        Ok(response_in_text)
    }

}