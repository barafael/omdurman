//! Bevy-side LLM glue: async task spawning and the `PendingCompletions`
//! resource. The pure transport (`LlmConfig`, `LlmError`,
//! `request_completion`) now lives in [`omdurman_net::llm`] and is re-exported
//! here so existing call sites (`telegram.rs`, `newspaper.rs`) are unchanged.
//!
//! Tasks run on Bevy's `IoTaskPool`, which has no Tokio reactor — reqwest
//! would panic at DNS resolution. The spawned body therefore calls the
//! blocking transport, which drives the request on its own runtime.

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};

pub use omdurman_net::llm::{LlmConfig, LlmError, request_completion_blocking};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionTag {
    Telegram { turn: u8 },
    Newspaper,
}

pub struct PendingCompletion {
    pub tag: CompletionTag,
    pub task: Task<Result<String, LlmError>>,
}

#[derive(Resource, Default)]
pub struct PendingCompletions {
    pub items: Vec<PendingCompletion>,
}

/// Token budgets for the flavour-text completions. A budget that is too small
/// does not fail the request — the API truncates the response at the cap
/// (`finish_reason: length`), which surfaced as newspaper articles ending
/// mid-sentence. Telegrams are one paragraph; the newspaper is a full
/// multi-paragraph article, so it gets the same budget the bot's LLM paths
/// use.
pub const TELEGRAM_MAX_TOKENS: u32 = 600;
pub const NEWSPAPER_MAX_TOKENS: u32 = 2000;

pub fn spawn_completion(
    cfg: &LlmConfig,
    sys: &str,
    usr: &str,
    tag: CompletionTag,
    max_tokens: u32,
    pending: &mut ResMut<PendingCompletions>,
) {
    if !cfg.has_key() {
        return;
    }
    let config = cfg.clone();
    let system = sys.to_string();
    let user = usr.to_string();

    let pool = IoTaskPool::get();
    let task =
        pool.spawn(async move { request_completion_blocking(&config, &system, &user, max_tokens) });

    pending.items.push(PendingCompletion { tag, task });
}
