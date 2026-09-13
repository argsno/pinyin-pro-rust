//! `customPinyin()` / `clearCustomDict()` ported from `lib/core/custom/`.

use crate::segmentit::{NewPattern, DICT_CUSTOM, PRIO_CUSTOM, PROB_CUSTOM};
use crate::store;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CustomHandle {
    Add,
    Replace,
}

#[derive(Clone, Debug, Default)]
pub struct CustomPinyinOptions {
    pub multiple: Option<CustomHandle>,
    pub polyphonic: Option<CustomHandle>,
}

fn char_count(s: &str) -> usize {
    s.chars().count()
}

/// Mirrors `customPinyin(config, options)`.
pub fn custom_pinyin(config: &[(String, String)], options: CustomPinyinOptions) {
    store::ensure_ac_built();
    // Merge into the retained custom word map, preserving first-insertion
    // order like the JS object (existing keys keep their position).
    let mut words = store::custom_words().write().unwrap();
    // Sort incoming words by length desc (stable), like the JS.
    let mut incoming: Vec<(String, String)> = config.to_vec();
    incoming.sort_by_key(|w| std::cmp::Reverse(char_count(&w.0)));
    for (w, p) in &incoming {
        match words.iter_mut().find(|(k, _)| k == w) {
            Some(slot) => slot.1 = p.clone(),
            None => words.push((w.clone(), p.clone())),
        }
    }
    let patterns: Vec<NewPattern> = words
        .iter()
        .map(|(w, p)| NewPattern {
            zh: w.clone(),
            pinyin: p.clone(),
            prob: PROB_CUSTOM + char_count(w) as f64,
            len: char_count(w),
            prio: PRIO_CUSTOM,
            dict: DICT_CUSTOM,
            pos: String::new(),
        })
        .collect();
    drop(words);
    let mut ac = store::ac().write().unwrap();
    ac.remove_dict(DICT_CUSTOM);
    ac.extend_trie(patterns);
    ac.build_fail();
    drop(ac);
    if let Some(h) = options.multiple {
        add_custom_to_dict(config, true, h);
    }
    if let Some(h) = options.polyphonic {
        add_custom_to_dict(config, false, h);
    }
}

fn add_custom_to_dict(config: &[(String, String)], multiple: bool, handle: CustomHandle) {
    for (word, pinyins) in config {
        let plist: Vec<&str> = pinyins.split(' ').collect();
        for (idx, ch) in word.chars().enumerate() {
            let chs = ch.to_string();
            let py = plist.get(idx).copied().unwrap_or("");
            let dict = if multiple {
                store::custom_multiple()
            } else {
                store::custom_polyphonic()
            };
            let mut d = dict.write().unwrap();
            let current = d.get_truthy(&chs);
            let base = current
                .clone()
                .or_else(|| store::dict1().read().unwrap().get_truthy(&chs));
            match handle {
                CustomHandle::Replace => {
                    d.set(&chs, py.to_string());
                }
                CustomHandle::Add => {
                    // addCustomConfigToDict 'add': supplement only.
                    // JS: if (!base) set(char, pinyin)
                    //     else { if (!current) set(char, base);
                    //            if (!base.split(' ').includes(pinyin)) set(base + pinyin) }
                    if let Some(merged) = base {
                        if current.is_none() {
                            d.set(&chs, merged.clone());
                        }
                        if !merged.split(' ').any(|x| x == py) {
                            let joined = if merged.is_empty() {
                                py.to_string()
                            } else if py.is_empty() {
                                merged
                            } else {
                                format!("{merged} {py}")
                            };
                            d.set(&chs, joined.trim().to_string());
                        }
                    } else {
                        d.set(&chs, py.to_string());
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CustomDictKind {
    Pinyin,
    Multiple,
    Polyphonic,
}

/// Mirrors `clearCustomDict(dict)`.
pub fn clear_custom_dict(kinds: &[CustomDictKind]) {
    for kind in kinds {
        match kind {
            CustomDictKind::Pinyin => {
                store::custom_words().write().unwrap().clear();
                store::ac().write().unwrap().remove_dict(DICT_CUSTOM);
            }
            CustomDictKind::Multiple => {
                store::custom_multiple().write().unwrap().clear();
            }
            CustomDictKind::Polyphonic => {
                store::custom_polyphonic().write().unwrap().clear();
            }
        }
    }
}
