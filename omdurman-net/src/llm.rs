//! Shared OpenAI-compatible LLM transport. Lives in the net glue crate so both
//! `omdurman-app` (flavour text: telegrams, newspapers) and `omdurman-bot`
//! (per-turn strategy advisor) reuse a single client. The Bevy-specific bits
//! (async task spawning, `PendingCompletions` resource) stay in the app; this
//! module owns only the pure transport: config, error type, and the
//! `request_completion` HTTP call.
//!
//! `LlmConfig` and `LlmError` are available on every target (the app inserts
//! `LlmConfig` as a Bevy resource on wasm too). The reqwest-backed
//! [`request_completion`] is native-only — reqwest + rustls do not build for
//! wasm32 — and is stubbed on wasm (the browser build withholds the API key in
//! [`LlmConfig::default`] anyway, so flavour-text systems degrade gracefully).

use bevy::prelude::Resource;

/// Configuration for an OpenAI-compatible chat-completions endpoint. Inserted as
/// a Bevy [`Resource`] by the app; constructed directly by the bot.
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

/// Transport / API error. Decoupled from `reqwest::Error` (stores the message)
/// so the type is identical on every target; only the native build can produce
/// [`LlmError::Reqwest`].
#[derive(Debug)]
pub enum LlmError {
    NoApiKey,
    Reqwest(String),
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

// --- native (reqwest-backed) transport --------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::{LlmConfig, LlmError};
    use serde::{Deserialize, Serialize};

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

    impl From<reqwest::Error> for LlmError {
        fn from(e: reqwest::Error) -> Self {
            LlmError::Reqwest(e.to_string())
        }
    }

    /// Send a single system+user chat-completion request and return the model's
    /// text response. Pure async — no Bevy. The app wraps this in an
    /// `IoTaskPool::spawn`; the bot calls it directly from its playthrough loop.
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
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::request_completion;

/// WASM stub: no HTTP client is available (reqwest + rustls don't build for
/// wasm32). The app's flavour-text systems call this and receive `NoApiKey`,
/// which is also what [`LlmConfig::default`] produces on wasm — so behaviour is
/// unchanged. The bot is native-only.
#[cfg(target_arch = "wasm32")]
pub async fn request_completion(
    _config: &LlmConfig,
    _system: &str,
    _user: &str,
    _max_tokens: u32,
) -> Result<String, LlmError> {
    Err(LlmError::NoApiKey)
}
