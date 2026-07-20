use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone)]
pub struct LlmConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let api_key = std::env::var("OPENAI_API_KEY").ok().filter(|k| !k.is_empty());
        #[cfg(target_arch = "wasm32")]
        let api_key = None;

        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
        }
    }
}

impl LlmConfig {
    pub fn has_key(&self) -> bool {
        self.api_key.as_ref().is_some_and(|k| !k.is_empty())
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

#[derive(Debug)]
pub enum LlmError {
    NoApiKey,
    Reqwest(reqwest::Error),
    Api(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NoApiKey => write!(f, "no API key configured"),
            LlmError::Reqwest(e) => write!(f, "HTTP error: {e}"),
            LlmError::Api(msg) => write!(f, "API error: {msg}"),
        }
    }
}

impl std::error::Error for LlmError {}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        LlmError::Reqwest(e)
    }
}

pub async fn request_completion(
    config: &LlmConfig,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String, LlmError> {
    let Some(api_key) = &config.api_key else {
        return Err(LlmError::NoApiKey);
    };

    let url = format!("{}/chat/completions", config.base_url);
    let body = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".into(),
                content: system.into(),
            },
            ChatMessage {
                role: "user".into(),
                content: user.into(),
            },
        ],
        max_tokens,
        temperature: 0.7,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(LlmError::Api(format!("HTTP {status}: {text}")));
    }

    let chat_resp: ChatResponse = resp.json().await?;
    let content = chat_resp
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();

    Ok(content)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionTag {
    Telegram { turn: u8 },
    Newspaper,
}

pub struct PendingCompletion {
    pub tag: CompletionTag,
    pub task: Task<Result<String, LlmError>>,
}

#[derive(Resource)]
#[derive(Default)]
pub struct PendingCompletions {
    pub items: Vec<PendingCompletion>,
}


pub fn spawn_completion(
    cfg: &LlmConfig,
    sys: &str,
    usr: &str,
    tag: CompletionTag,
    pending: &mut ResMut<PendingCompletions>,
) {
    if !cfg.has_key() {
        return;
    }
    let config = cfg.clone();
    let system = sys.to_string();
    let user = usr.to_string();

    let pool = IoTaskPool::get();
    let task = pool.spawn(async move {
        request_completion(&config, &system, &user, 300).await
    });

    pending.items.push(PendingCompletion { tag, task });
}
