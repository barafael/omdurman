//! Event-source serialization format — pure serde, no logic.
//!
//! Defines the [`GameEvent`] enum and RON serialization. This is the stable
//! contract shared by `omdurman-rules`, `omdurman-net`, `omdurman-app`,
//! save files, and replay logs.
