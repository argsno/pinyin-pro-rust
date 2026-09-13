//! `addDict()` / `removeDict()` ported from `lib/core/dict/index.ts`.

use crate::segmentit::{NewPattern, PRIO_NORMAL, PROB_DICT};
use crate::store::{self, DEFAULT_DICT_NAME};

#[derive(Clone, Debug)]
pub struct DictValue {
    pub pinyin: String,
    pub probability: Option<f64>,
    pub pos: Option<String>,
    /// Whether the entry came as an array (single-char entries always
    /// register a word pattern in that case, even for length 1).
    pub is_array: bool,
}

impl From<&str> for DictValue {
    fn from(pinyin: &str) -> Self {
        DictValue {
            pinyin: pinyin.to_string(),
            probability: None,
            pos: None,
            is_array: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Dict1Handle {
    #[default]
    Add,
    Replace,
    Ignore,
}

#[derive(Clone, Debug)]
pub struct AddDictOptions {
    pub name: Option<String>,
    pub dict1: Dict1Handle,
}

impl Default for AddDictOptions {
    fn default() -> Self {
        AddDictOptions {
            name: None,
            dict1: Dict1Handle::Add,
        }
    }
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// Mirrors `addDict(dict, options)`.
pub fn add_dict(dict: &[(String, DictValue)], options: AddDictOptions) {
    store::ensure_ac_built();
    let dict_name = options
        .name
        .unwrap_or_else(|| DEFAULT_DICT_NAME.to_string());
    let dict_id = store::named_dict_id(&dict_name);
    let mut patterns = Vec::with_capacity(dict.len());
    for (word, value) in dict {
        let len = char_count(word);
        if len == 1 {
            add_to_origin_dict(dict_id, word, &value.pinyin, options.dict1);
        }
        // String values always push a pattern too (JS pushes for both forms).
        let prob = value
            .probability
            .unwrap_or(PROB_DICT * len as f64 * len as f64);
        patterns.push(NewPattern {
            zh: word.clone(),
            pinyin: value.pinyin.clone(),
            prob,
            len,
            prio: PRIO_NORMAL,
            dict: dict_id,
            pos: value.pos.clone().unwrap_or_default(),
        });
    }
    let mut ac = store::ac().write().unwrap();
    ac.extend_trie(patterns);
    ac.build_fail();
}

/// Mirrors `removeDict(dictName?)`.
pub fn remove_dict(name: Option<&str>) {
    let dict_name = name.unwrap_or(DEFAULT_DICT_NAME);
    let dict_id = store::named_dict_id(dict_name);
    store::ac().write().unwrap().remove_dict(dict_id);
    // Restore single-char readings saved by add_to_origin_dict.
    let mut origins = store::origin_dict().write().unwrap();
    if let Some(saved) = origins.remove(&dict_id) {
        let mut d = store::dict1().write().unwrap();
        for (ch, original) in saved {
            match original {
                Some(p) => d.set(&ch, p),
                None => d.remove(&ch),
            }
        }
    }
}

fn add_to_origin_dict(dict_id: u64, ch: &str, pinyin: &str, handle: Dict1Handle) {
    let mut origins = store::origin_dict().write().unwrap();
    let entry = origins.entry(dict_id).or_default();
    entry
        .entry(ch.to_string())
        .or_insert_with(|| store::dict1().read().unwrap().get_truthy(ch));
    let mut d = store::dict1().write().unwrap();
    match handle {
        Dict1Handle::Add => {
            let existed = d.get_truthy(ch);
            match existed {
                Some(e) => {
                    if !e.split(' ').any(|x| x == pinyin) {
                        d.set(ch, format!("{e} {pinyin}"));
                    }
                }
                None => {
                    d.set(ch, pinyin.to_string());
                }
            }
        }
        Dict1Handle::Replace => {
            d.set(ch, pinyin.to_string());
        }
        Dict1Handle::Ignore => {}
    }
}
