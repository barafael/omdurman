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

/// Optional structured-output hint sent as the chat-completions
/// `response_format` field. `JsonObject` asks the model to emit valid JSON
/// (`{"type": "json_object"}`); most OpenAI-compatible endpoints honour it.
/// `None` omits the field, so callers that want plain prose (the app's
/// telegram/newspaper flavour text) are unaffected.
#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    JsonObject,
}

/// Configuration for an OpenAI-compatible chat-completions endpoint. Inserted as
/// a Bevy [`Resource`] by the app; constructed directly by the bot.
#[derive(Resource, Clone)]
pub struct LlmConfig {
    pub api_key: Option<String>,
    pub base_url: String,
    pub model: String,
    /// Optional `response_format` hint. Defaults to `None` (plain prose);
    /// set via [`LlmConfig::with_json_object`] where the reply must be JSON.
    pub response_format: Option<ResponseFormat>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        fn env_nonempty(name: &str) -> Option<String> {
            std::env::var(name).ok().filter(|v| !v.is_empty())
        }
        #[cfg(target_arch = "wasm32")]
        fn env_nonempty(_name: &str) -> Option<String> {
            None
        }

        Self {
            // `LLM_API_KEY` wins so any OpenAI-compatible provider shares one
            // token across the app and the bot; `OPENAI_API_KEY` is the fallback.
            api_key: env_nonempty("LLM_API_KEY").or_else(|| env_nonempty("OPENAI_API_KEY")),
            base_url: env_nonempty("LLM_BASE_URL")
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model: env_nonempty("LLM_MODEL").unwrap_or_else(|| "gpt-4o-mini".to_string()),
            response_format: None,
        }
    }
}

impl LlmConfig {
    pub fn has_key(&self) -> bool {
        self.api_key.as_ref().is_some_and(|k| !k.is_empty())
    }

    /// Ask the endpoint to return valid JSON (`response_format` = json_object).
    pub fn with_json_object(mut self) -> Self {
        self.response_format = Some(ResponseFormat::JsonObject);
        self
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
        #[serde(skip_serializing_if = "Option::is_none")]
        response_format: Option<super::ResponseFormat>,
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
            response_format: config.response_format,
        };

        // reqwest is built with `rustls-no-provider` (see Cargo.toml): matchbox
        // enables rustls's `ring`, but reqwest only auto-picks a provider via
        // its own `__rustls-aws-lc-rs` feature and otherwise panics unless a
        // process-wide default is installed. Install ring explicitly — the
        // same (and only) provider in the graph, so WebRTC DTLS keeps using
        // it and no second provider is introduced.
        let _ = rustls::crypto::ring::default_provider().install_default();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(90))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
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

    /// See the re-export site ([`crate::llm::request_completion_blocking`])
    /// for rationale.
    pub fn request_completion_blocking(
        config: &LlmConfig,
        system: &str,
        user: &str,
        max_tokens: u32,
    ) -> Result<String, LlmError> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime")
            .block_on(request_completion(config, system, user, max_tokens))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::request_completion;

/// Blocking variant of [`request_completion`] for executors without a Tokio
/// reactor (Bevy's `IoTaskPool`): drives the async transport on a dedicated
/// current-thread runtime. One runtime per call — these are rare, user-facing
/// flavour-text requests. The bot CLI keeps its own runtime and uses the
/// async form.
#[cfg(not(target_arch = "wasm32"))]
pub use native::request_completion_blocking;

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

/// WASM stub mirroring the async one above (see `request_completion`).
#[cfg(target_arch = "wasm32")]
pub fn request_completion_blocking(
    _config: &LlmConfig,
    _system: &str,
    _user: &str,
    _max_tokens: u32,
) -> Result<String, LlmError> {
    Err(LlmError::NoApiKey)
}
