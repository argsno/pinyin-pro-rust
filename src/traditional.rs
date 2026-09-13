//! `addTraditionalDict()` / `getTraditionalDict()` ported from
//! `lib/core/traditional/index.ts`.

use std::collections::HashMap;

use crate::store;

/// Mirrors `addTraditionalDict(dict)`: keys are single chars (JS uses the
/// first UTF-16 unit of each key).
pub fn add_traditional_dict(dict: &[(String, String)]) {
    let mut t = store::traditional().write().unwrap();
    for (key, value) in dict {
        if let (Some(k), Some(v)) = (key.chars().next(), value.chars().next()) {
            t.insert(k, v);
        }
    }
}

/// Snapshot of the current traditional map.
pub fn get_traditional_dict() -> HashMap<char, char> {
    store::traditional().read().unwrap().clone()
}
