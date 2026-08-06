//! Bevy-side LLM glue: async task spawning and the `PendingCompletions`
//! resource. The pure transport (`LlmConfig`, `LlmError`,
//! `request_completion`) now lives in [`omdurman_net::llm`] and is re-exported
//! here so existing call sites (`telegram.rs`, `newspaper.rs`) are unchanged.

use bevy::prelude::*;
use bevy::tasks::{IoTaskPool, Task};

pub use omdurman_net::llm::{LlmConfig, LlmError, request_completion};

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
