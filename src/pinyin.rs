//! Core `pinyin()` ported from `lib/core/pinyin/`.

use regex::Regex;

use crate::options::{
    AllInfo, NonZh, PatternKind, PinyinOptions, PinyinOutput, SurnameMode, ToneType, TypeMode,
    VMode,
};
use crate::pinyin_utils::{
    final_parts, final_parts_from_final, first_letter, initial_and_final, num_of_tone,
    pinyin_with_num, strip_tone,
};
use crate::segmentit::MatchPattern;
use crate::store;

// ---------------------------------------------------------------------------
// word list items
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Sword {
    pub origin: String,
    pub origin_pinyin: String,
    pub result: String,
    pub is_zh: bool,
    pub deleted: bool,
}

impl Sword {
    pub(crate) fn new(origin: String, result: String, is_zh: bool) -> Self {
        let origin_pinyin = result.clone();
        Sword {
            origin,
            origin_pinyin,
            result,
            is_zh,
            deleted: false,
        }
    }
}

// ---------------------------------------------------------------------------
// single-char lookups
// ---------------------------------------------------------------------------

/// First (most common) reading of a char, or the char itself.
pub fn get_single_word_pinyin(ch: &str) -> String {
    match store::dict1().read().unwrap().get_truthy(ch) {
        Some(p) => match p.find(' ') {
            Some(i) => p[..i].to_string(),
            None => p,
        },
        None => ch.to_string(),
    }
}

/// All readings of a char (for `multiple`, `match`, `type: all`).
pub fn get_all_pinyin(ch: &str, surname: SurnameMode) -> Vec<String> {
    if let Some(p) = store::custom_multiple().read().unwrap().get_truthy(ch) {
        return p.split(' ').map(|s| s.to_string()).collect();
    }
    let mut readings: Vec<String> = store::dict1()
        .read()
        .unwrap()
        .get_truthy(ch)
        .map(|p| p.split(' ').map(|s| s.to_string()).collect())
        .unwrap_or_default();
    if surname != SurnameMode::Off {
        if let Some(sp) = surname_reading(ch) {
            readings.retain(|r| r != &sp);
            readings.insert(0, sp);
        }
    }
    readings
}

fn surname_reading(ch: &str) -> Option<String> {
    // Surname readings live in the word-pattern table; single-char surnames
    // are looked up from the generated surname list via a lazy map.
    use std::collections::HashMap;
    use std::sync::OnceLock;
    static MAP: OnceLock<HashMap<String, String>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for p in crate::data::PATTERNS_SURNAME {
            if p.zh.chars().count() == 1 {
                m.entry(p.zh.to_string())
                    .or_insert_with(|| p.pinyin.to_string());
            }
        }
        m
    });
    map.get(ch).cloned()
}

// ---------------------------------------------------------------------------
// tone sandhi + special single-char handling
// ---------------------------------------------------------------------------

fn process_tone_sandhi(cur: &str, pre: Option<&str>, next: Option<&str>) -> Option<String> {
    use crate::data::special::{
        TONE_SANDHI_BU, TONE_SANDHI_IGNORE_BU, TONE_SANDHI_IGNORE_YI, TONE_SANDHI_YI,
    };
    let (table, ignore): (&[(&str, &[i64])], &[&str]) = match cur {
        "不" => (TONE_SANDHI_BU, TONE_SANDHI_IGNORE_BU),
        "一" => (TONE_SANDHI_YI, TONE_SANDHI_IGNORE_YI),
        _ => return None,
    };
    // 叠词轻声: 说不说 / 说一说
    if let (Some(pre), Some(next)) = (pre, next) {
        if pre == next && !pre.is_empty() && get_single_word_pinyin(pre) != pre {
            return Some(strip_tone(&get_single_word_pinyin(cur)));
        }
    }
    if let Some(next) = next {
        if !next.is_empty() && !ignore.contains(&next) {
            let next_py = get_single_word_pinyin(next);
            if next_py != next {
                if let Ok(tone) = num_of_tone(&next_py).parse::<i64>() {
                    for (reading, tones) in table {
                        if tones.contains(&tone) {
                            return Some(reading.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

fn process_tone_sandhi_liao(cur: &str, pre: Option<&str>) -> Option<String> {
    if cur == "了" {
        let need = match pre {
            None => true,
            Some(p) => store::dict1().read().unwrap().get_truthy(p).is_none(),
        };
        if need {
            return Some("liǎo".to_string());
        }
    }
    None
}

fn process_reduplication(cur: &str, pre: Option<&str>) -> Option<String> {
    if cur == "々" {
        match pre {
            None => return Some("tóng".to_string()),
            Some(p) => match store::dict1().read().unwrap().get_truthy(p) {
                None => return Some("tóng".to_string()),
                Some(pp) => {
                    return Some(match pp.find(' ') {
                        Some(i) => pp[..i].to_string(),
                        None => pp,
                    })
                }
            },
        }
    }
    None
}

fn process_special(
    cur: &str,
    pre: Option<&str>,
    next: Option<&str>,
    has_reduplication: bool,
) -> String {
    if has_reduplication {
        if let Some(r) = process_reduplication(cur, pre) {
            return r;
        }
    }
    if let Some(r) = process_tone_sandhi_liao(cur, pre) {
        return r;
    }
    if let Some(r) = process_tone_sandhi(cur, pre, next) {
        return r;
    }
    get_single_word_pinyin(cur)
}

// ---------------------------------------------------------------------------
// main segmentation -> word list
// ---------------------------------------------------------------------------

fn traditional_map(word: &[char]) -> Vec<char> {
    let dict = store::traditional().read().unwrap();
    word.iter()
        .map(|&c| dict.get(&c).copied().unwrap_or(c))
        .collect()
}

/// Mirrors `getPinyin()`: returns the per-char list plus resolved matches
/// (used by `segment()` for grouping).
pub fn get_pinyin(
    word_chars: &[char],
    surname: SurnameMode,
    traditional: bool,
) -> (Vec<Sword>, Vec<MatchPattern>) {
    get_pinyin_with_algo(
        word_chars,
        surname,
        crate::segmentit::TokenizationAlgorithm::default(),
        traditional,
    )
}

pub fn get_pinyin_with_algo(
    word_chars: &[char],
    surname: SurnameMode,
    algo: crate::segmentit::TokenizationAlgorithm,
    traditional: bool,
) -> (Vec<Sword>, Vec<MatchPattern>) {
    use crate::segmentit::PRIO_NORMAL;
    let search_chars: Vec<char> = if traditional {
        traditional_map(word_chars)
    } else {
        word_chars.to_vec()
    };
    let ac = store::ac();
    let matches = ac.read().unwrap().search(&search_chars, surname, algo);
    let has_reduplication = word_chars.contains(&'々');
    let mut list: Vec<Option<Sword>> = vec![None; word_chars.len()];
    let mut resolved: Vec<MatchPattern> = Vec::with_capacity(matches.len());
    let mut mi = 0;
    let mut i = 0;
    while i < word_chars.len() {
        let m = matches.get(mi);
        let at_match = m.map(|m| m.index == i).unwrap_or(false);
        if at_match {
            let mut m = matches[mi].clone();
            if m.len == 1 && m.prio <= PRIO_NORMAL {
                let ch: String = word_chars[i].to_string();
                m.zh = ch.clone();
                let pre = if i > 0 {
                    Some(word_chars[i - 1].to_string())
                } else {
                    None
                };
                let next = word_chars.get(i + 1).map(|c| c.to_string());
                let p = process_special(&ch, pre.as_deref(), next.as_deref(), has_reduplication);
                let is_zh = p != ch;
                list[i] = Some(Sword::new(ch, p, is_zh));
                i += 1;
                resolved.push(m);
                mi += 1;
                continue;
            }
            let pinyins: Vec<&str> = m.pinyin.split(' ').collect();
            if traditional {
                m.zh = word_chars[m.index..m.index + m.len].iter().collect();
            }
            for (j, p) in pinyins.iter().enumerate().take(m.len) {
                let ch: String = word_chars[m.index + j].to_string();
                list[i + j] = Some(Sword::new(ch, p.to_string(), true));
            }
            // Defensive: data guarantees pinyins.len() >= m.len.
            for j in pinyins.len()..m.len {
                let ch: String = word_chars[m.index + j].to_string();
                list[i + j] = Some(Sword::new(ch, String::new(), true));
            }
            i += m.len;
            resolved.push(m);
            mi += 1;
        } else {
            let ch: String = word_chars[i].to_string();
            let pre = if i > 0 {
                Some(word_chars[i - 1].to_string())
            } else {
                None
            };
            let next = word_chars.get(i + 1).map(|c| c.to_string());
            let p = process_special(&ch, pre.as_deref(), next.as_deref(), has_reduplication);
            let is_zh = p != ch;
            list[i] = Some(Sword::new(ch, p, is_zh));
            i += 1;
        }
    }
    (
        list.into_iter()
            .map(|o| o.expect("word list hole"))
            .collect(),
        resolved,
    )
}

// ---------------------------------------------------------------------------
// middlewares
// ---------------------------------------------------------------------------

fn in_non_zh_scope(origin: &str, scope: &Option<Regex>) -> bool {
    match scope {
        None => true,
        Some(re) => re.is_match(origin),
    }
}

pub fn compile_scope(pattern: &Option<String>) -> Option<Regex> {
    // JS tests the regex per single char; global/sticky lastIndex is reset.
    // Compile once; `is_match` on a one-char haystack is equivalent.
    pattern.as_deref().and_then(|p| Regex::new(p).ok())
}

pub fn middleware_non_zh(list: Vec<Sword>, non_zh: NonZh, scope: &Option<Regex>) -> Vec<Sword> {
    match non_zh {
        NonZh::Removed => list
            .into_iter()
            .filter(|item| item.is_zh || !in_non_zh_scope(&item.origin, scope))
            .collect(),
        NonZh::Consecutive => {
            let mut list = list;
            for i in (0..list.len().saturating_sub(1)).rev() {
                // Mirrors the JS exactly: the absorbed item keeps is_zh=false and
                // participates in further merges via its accumulated strings; the
                // `deleted` flag only filters it out at the end.
                let merge = !list[i].is_zh
                    && !list[i + 1].is_zh
                    && in_non_zh_scope(&list[i].origin.clone(), scope)
                    && in_non_zh_scope(&list[i + 1].origin.clone(), scope);
                if merge {
                    let next_origin = list[i + 1].origin.clone();
                    let next_result = list[i + 1].result.clone();
                    list[i].origin.push_str(&next_origin);
                    list[i].result.push_str(&next_result);
                    list[i + 1].deleted = true;
                }
            }
            list.into_iter().filter(|item| !item.deleted).collect()
        }
        NonZh::Spaced => list,
    }
}

fn get_multiple_pinyin(word_chars: &[char], surname: SurnameMode) -> Vec<Sword> {
    let word: String = word_chars.iter().collect();
    let readings = get_all_pinyin(&word, surname);
    if readings.is_empty() {
        vec![Sword::new(word.clone(), word.clone(), false)]
    } else {
        readings
            .into_iter()
            .map(|p| Sword::new(word.clone(), p, true))
            .collect()
    }
}

pub fn middleware_multiple(
    word_chars: &[char],
    multiple: bool,
    surname: SurnameMode,
) -> Option<Vec<Sword>> {
    if multiple && word_chars.len() == 1 {
        Some(get_multiple_pinyin(word_chars, surname))
    } else {
        None
    }
}

pub fn middleware_pattern(list: &mut [Sword], opts: &PinyinOptions) {
    match opts.pattern {
        PatternKind::Pinyin => {}
        PatternKind::Num => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    num_of_tone(&item.result)
                } else {
                    String::new()
                };
            }
        }
        PatternKind::Initial => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    initial_and_final(&item.result, opts.initial_pattern).0
                } else {
                    String::new()
                };
            }
        }
        PatternKind::Final => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    initial_and_final(&item.result, opts.initial_pattern).1
                } else {
                    String::new()
                };
            }
        }
        PatternKind::First => {
            for item in list.iter_mut() {
                item.result = first_letter(&item.result, item.is_zh);
            }
        }
        PatternKind::FinalHead => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    final_parts(&item.result).0
                } else {
                    String::new()
                };
            }
        }
        PatternKind::FinalBody => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    final_parts(&item.result).1
                } else {
                    String::new()
                };
            }
        }
        PatternKind::FinalTail => {
            for item in list.iter_mut() {
                item.result = if item.is_zh {
                    final_parts(&item.result).2
                } else {
                    String::new()
                };
            }
        }
    }
}

pub fn middleware_tone_type(list: &mut [Sword], opts: &PinyinOptions) {
    match opts.tone_type {
        ToneType::Symbol => {}
        ToneType::None => {
            for item in list.iter_mut() {
                if item.is_zh {
                    item.result = strip_tone(&item.result);
                }
            }
        }
        ToneType::Num => {
            for item in list.iter_mut() {
                if item.is_zh {
                    item.result = pinyin_with_num(&item.result, &item.origin_pinyin.clone());
                }
            }
        }
    }
}

pub fn middleware_v(list: &mut [Sword], v: &VMode) {
    match v {
        VMode::Off => {}
        VMode::V => {
            for item in list.iter_mut() {
                if item.is_zh {
                    item.result = item.result.replace('ü', "v");
                }
            }
        }
        VMode::Custom(rep) => {
            for item in list.iter_mut() {
                if item.is_zh {
                    item.result = item.result.replace('ü', rep.as_str());
                }
            }
        }
    }
}

pub fn middleware_tone_sandhi(list: &mut [Sword], tone_sandhi: bool) {
    if !tone_sandhi {
        for item in list.iter_mut() {
            if item.origin == "一" {
                item.result = "yī".to_string();
                item.origin_pinyin = "yī".to_string();
            } else if item.origin == "不" {
                item.result = "bù".to_string();
                item.origin_pinyin = "bù".to_string();
            }
        }
    }
}

fn middleware_type(
    mut list: Vec<Sword>,
    opts: &PinyinOptions,
    word_chars: &[char],
) -> PinyinOutput {
    if opts.multiple && word_chars.len() == 1 {
        let mut last = String::new();
        list.retain(|item| {
            let keep = item.result != last;
            last = item.result.clone();
            keep
        });
    }
    match opts.type_mode {
        TypeMode::Array => PinyinOutput::Arr(list.into_iter().map(|i| i.result).collect()),
        TypeMode::All => PinyinOutput::All(
            list.iter()
                .map(|item| {
                    let pinyin = if item.is_zh {
                        item.result.clone()
                    } else {
                        String::new()
                    };
                    let (initial, fin) = initial_and_final(&pinyin, opts.initial_pattern);
                    let (head, body, tail) = final_parts_from_final(&fin);
                    let mut polyphonic = Vec::new();
                    if !pinyin.is_empty() {
                        polyphonic.push(pinyin.clone());
                        for alt in get_all_pinyin(&item.origin, opts.surname()) {
                            if alt != pinyin {
                                polyphonic.push(alt);
                            }
                        }
                    }
                    AllInfo {
                        origin: item.origin.clone(),
                        pinyin: pinyin.clone(),
                        initial,
                        final_: fin,
                        first: first_letter(&item.result, item.is_zh),
                        final_head: head,
                        final_body: body,
                        final_tail: tail,
                        num: num_of_tone(&item.origin_pinyin).parse::<i64>().unwrap_or(0),
                        is_zh: item.is_zh,
                        polyphonic,
                        in_zh_range: store::dict1()
                            .read()
                            .unwrap()
                            .get_truthy(&item.origin)
                            .is_some(),
                        result: item.result.clone(),
                    }
                })
                .collect(),
        ),
        TypeMode::Str => PinyinOutput::Str(
            list.into_iter()
                .map(|i| i.result)
                .collect::<Vec<_>>()
                .join(&opts.separator),
        ),
    }
}

// ---------------------------------------------------------------------------
// public entry
// ---------------------------------------------------------------------------

/// Convert Chinese text to pinyin. Mirrors `pinyin(word, options)`.
pub fn pinyin(text: &str, options: PinyinOptions) -> PinyinOutput {
    let opts = options.normalized();
    if text.is_empty() {
        return match opts.type_mode {
            TypeMode::Str => PinyinOutput::Str(String::new()),
            TypeMode::Array => PinyinOutput::Arr(Vec::new()),
            TypeMode::All => PinyinOutput::All(Vec::new()),
        };
    }
    let word_chars: Vec<char> = text.chars().collect();
    let scope = compile_scope(&opts.non_zh_scope);
    let (list0, _) = get_pinyin_with_algo(
        &word_chars,
        opts.surname(),
        opts.segmentit,
        opts.traditional,
    );
    // 一/不 tone-sandhi toggle
    let mut list = list0;
    middleware_tone_sandhi(&mut list, opts.tone_sandhi);
    // nonZh
    list = middleware_non_zh(list, opts.non_zh, &scope);
    // multiple
    if let Some(m) = middleware_multiple(&word_chars, opts.multiple, opts.surname()) {
        list = m;
    }
    middleware_pattern(&mut list, &opts);
    middleware_tone_type(&mut list, &opts);
    middleware_v(&mut list, &opts.v);
    middleware_type(list, &opts, &word_chars)
}
