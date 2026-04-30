pub mod gemini;

#[derive(Debug)]
pub struct LlmRequest {
    pub contents: Vec<LlmContent>,
}

#[derive(Debug)]
pub enum LlmContent {
    User(String),
    Assistant(String),
}
