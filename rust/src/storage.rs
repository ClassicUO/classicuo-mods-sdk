//! Typed sugar over `host::storage_get` / `host::storage_set`: one JSON blob per mod,
//! global or per character. Writes go to disk immediately — save on a user action,
//! not every frame.

use crate::host::{self, Scope};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// The stored blob parsed as `T`; `None` when nothing is stored or it doesn't parse.
pub fn load<T: DeserializeOwned>(scope: Scope) -> Option<T> {
    let raw = host::storage_get(scope);
    if raw.is_empty() {
        return None;
    }
    serde_json::from_str(&raw).ok()
}

/// Stores `value` as the scope's blob.
pub fn save<T: Serialize>(scope: Scope, value: &T) {
    host::storage_set(scope, &serde_json::to_string(value).expect("storage serialize"));
}
