//! Static dictionary data: single-char readings, word patterns, special tables.
//!
//! `dict1_data.rs` / `patterns_data.rs` are auto-generated from the upstream
//! `packages/pinyin-pro/lib/data/*.ts` sources.
//! The number-word patterns mirror `genNumberDict()` in `special.ts`.

mod dict1_data;
mod patterns_data;
pub mod special;

pub use dict1_data::DICT1_ENTRIES;
pub use patterns_data::{
    surname_prob, PATTERNS2, PATTERNS3, PATTERNS4, PATTERNS5, PATTERNS_SURNAME,
};

use crate::segmentit::{NewPattern, DICT_BUILTIN, PRIO_NORMAL, PROB_DICT, PROB_RULE};

/// All built-in word patterns in `PatternsNormal` order:
/// dict5, dict4, dict3, dict2, number rules, surnames.
pub fn builtin_patterns() -> Vec<NewPattern> {
    let mut out = Vec::with_capacity(3600);
    for p in PATTERNS5
        .iter()
        .chain(PATTERNS4.iter())
        .chain(PATTERNS3.iter())
        .chain(PATTERNS2.iter())
    {
        out.push(NewPattern {
            zh: p.zh.to_string(),
            pinyin: p.pinyin.to_string(),
            prob: PROB_DICT,
            len: p.len,
            prio: PRIO_NORMAL,
            dict: DICT_BUILTIN,
            pos: String::new(),
        });
    }
    for (zh, pinyin) in number_entries() {
        out.push(NewPattern {
            zh,
            pinyin,
            prob: PROB_RULE,
            len: 0, // recomputed below; placeholder replaced at trie build
            prio: PRIO_NORMAL,
            dict: DICT_BUILTIN,
            pos: String::new(),
        });
    }
    for p in PATTERNS_SURNAME.iter() {
        out.push(NewPattern {
            zh: p.zh.to_string(),
            pinyin: p.pinyin.to_string(),
            prob: surname_prob(p.len),
            len: p.len,
            prio: crate::segmentit::PRIO_SURNAME,
            dict: DICT_BUILTIN,
            pos: String::new(),
        });
    }
    for p in out.iter_mut() {
        if p.len == 0 {
            p.len = p.zh.chars().count();
        }
    }
    out
}

/// (zh, pinyin) pairs from `genNumberDict()`: fixed entries first, then every
/// `Numbers × NumberWordMap` combination in JS object iteration order.
fn number_entries() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let fixed: &[(&str, &str)] = &[
        ("零一", "líng yī"),
        ("〇一", "líng yī"),
        ("十一", "shí yī"),
        ("一十", "yī shí"),
        ("第一", "dì yī"),
        ("一十一", "yī shí yī"),
    ];
    for (zh, pinyin) in fixed {
        out.push((zh.to_string(), pinyin.to_string()));
    }
    for (num, num_py) in special::NUMBERS {
        for (word, word_py) in special::NUMBER_WORD_MAP {
            out.push((format!("{num}{word}"), format!("{num_py} {word_py}")));
        }
    }
    out
}
