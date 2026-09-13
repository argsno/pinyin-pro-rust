//! `match()` ported from `lib/core/match/index.ts`.
//!
//! Note: returned indices are Unicode scalar (char) positions. For BMP-only
//! text they equal the JS UTF-16 indices; characters outside the BMP count
//! as one position (the JS `processDoubleUnicodeIndex` adjustment has no
//! counterpart since Rust strings cannot hold lone surrogates).

use crate::options::SurnameMode;
use crate::options::{MatchOptions, MatchPrecision, MatchSpace, VMode};
use crate::pinyin::get_all_pinyin;

const MAX_PINYIN_LENGTH: usize = 6;

fn tone_base(c: char) -> Option<char> {
    match c {
        'ā' | 'á' | 'ǎ' | 'à' => Some('a'),
        'ō' | 'ó' | 'ǒ' | 'ò' => Some('o'),
        'ē' | 'é' | 'ě' | 'è' => Some('e'),
        'ī' | 'í' | 'ǐ' | 'ì' => Some('i'),
        'ū' | 'ú' | 'ǔ' | 'ù' => Some('u'),
        'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' => Some('ü'),
        'ń' | 'ň' | 'ǹ' => Some('n'),
        'ḿ' => Some('m'),
        'ế' | 'ề' => Some('ê'),
        _ => None,
    }
}

fn is_combining_mark(c: char) -> bool {
    matches!(c, '\u{300}' | '\u{301}' | '\u{304}' | '\u{30c}')
}

/// Strip tones like the match module's TONE_MAP (incl. decomposed n/m/ê).
fn strip_tone(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if i + 1 < chars.len()
            && ((c == 'n' || c == 'm' || c == 'ê') && is_combining_mark(chars[i + 1]))
        {
            out.push(c);
            i += 2;
            continue;
        }
        match tone_base(c) {
            Some(b) => out.push(b),
            None => out.push(c),
        }
        i += 1;
    }
    out
}

fn apply_v(s: &str, v: &VMode) -> String {
    match v {
        VMode::Off => s.to_string(),
        VMode::V => s.replace('ü', "v"),
        VMode::Custom(rep) => s.replace('ü', rep.as_str()),
    }
}

fn match_pinyin_for(ch: &str, options: &MatchOptions) -> Vec<String> {
    let readings = get_all_pinyin(ch, SurnameMode::Off);
    let base = if readings.is_empty() {
        vec![ch.to_string()]
    } else {
        readings
    };
    base.into_iter()
        .map(|p| apply_v(&strip_tone(&p), &options.v))
        .collect()
}

/// Mirrors `match(text, pinyin, options)`.
/// Returns char indices of matched chars, or `None`.
pub fn match_text(text: &str, pinyin: &str, options: MatchOptions) -> Option<Vec<usize>> {
    let mut options = options;
    if options.precision == MatchPrecision::Any {
        options.last_precision = MatchPrecision::Any;
    }
    let mut query = if options.v != VMode::Off {
        apply_v(pinyin, &options.v)
    } else {
        pinyin.to_string()
    };
    let mut text_owned = text.to_string();
    if options.insensitive {
        text_owned = text_owned.to_lowercase();
        query = query.to_lowercase();
    }
    if options.space == MatchSpace::Ignore {
        query = query.chars().filter(|c| !c.is_whitespace()).collect();
    }
    if options.precision == MatchPrecision::Any {
        match_any(&text_owned, &query, &options)
    } else {
        match_above_start(&text_owned, &query, &options)
    }
}

fn match_length(p1: &str, p2: &str) -> usize {
    // characters of p1 matched in order against p2's prefix
    let p2: Vec<char> = p2.chars().collect();
    let mut len = 0;
    for c in p1.chars() {
        if len < p2.len() && c == p2[len] {
            len += 1;
        }
    }
    len
}

fn match_any(text: &str, pinyin: &str, options: &MatchOptions) -> Option<Vec<usize>> {
    let words: Vec<char> = text.chars().collect();
    let mut query: Vec<char> = pinyin.chars().collect();
    let ignore_space = options.space == MatchSpace::Ignore;
    let mut result = Vec::new();
    for (i, &w) in words.iter().enumerate() {
        if ignore_space && w == ' ' {
            result.push(i);
            continue;
        }
        if !query.is_empty() && w == query[0] {
            query.remove(0);
            result.push(i);
            continue;
        }
        let ps = match_pinyin_for(&w.to_string(), options);
        let mut best = 0;
        for p in &ps {
            let l = match_length(p, &query.iter().collect::<String>());
            if l > best {
                best = l;
            }
        }
        if best > 0 {
            query.drain(..best);
            result.push(i);
        }
        if query.is_empty() {
            break;
        }
    }
    if !query.is_empty() {
        return None;
    }
    if options.continuous && result.windows(2).any(|w| w[1] != w[0] + 1) {
        return None;
    }
    if ignore_space {
        result.retain(|&i| words[i] != ' ');
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn match_above_start(text: &str, pinyin: &str, options: &MatchOptions) -> Option<Vec<usize>> {
    let words: Vec<char> = text.chars().collect();
    let query: Vec<char> = pinyin.chars().collect();
    // dp rows of Option<Vec<usize>>; None = unreachable
    let mut pre: Vec<Option<Vec<usize>>> = vec![None; query.len() + 1];
    pre[0] = Some(Vec::new());

    for i in 1..=words.len() {
        let mut current: Vec<Option<Vec<usize>>> = vec![None; query.len() + 1];
        current[0] = Some(Vec::new());
        if !options.continuous || (options.space == MatchSpace::Ignore && words[i - 1] == ' ') {
            let n = query.len();
            current[..n].clone_from_slice(&pre[..n]);
        }
        let mut muls: Option<Vec<String>> = None;
        for j in 1..=query.len() {
            let reachable = match &pre[j - 1] {
                None => false,
                Some(v) => j == 1 || !v.is_empty(),
            };
            if !reachable {
                continue;
            }
            if muls.is_none() {
                muls = Some(match_pinyin_for(&words[i - 1].to_string(), options));
            }
            let muls = muls.as_ref().unwrap();
            // non-Chinese direct char match
            if words[i - 1] == query[j - 1] {
                let mut m = pre[j - 1].clone().unwrap();
                m.push(i - 1);
                if current[j].as_ref().map(|v| v.len()).unwrap_or(0) < m.len() {
                    current[j] = Some(m);
                }
                if j == query.len() {
                    return current[j].clone();
                }
            }
            // last-char handling
            if query.len() - j <= MAX_PINYIN_LENGTH {
                let rest: String = query[j - 1..].iter().collect();
                let last = muls.iter().any(|py| match options.last_precision {
                    MatchPrecision::Any => py.contains(&rest),
                    MatchPrecision::Start => py.starts_with(&rest),
                    MatchPrecision::First => {
                        py.chars().next().map(|c| c.to_string()).unwrap_or_default() == rest
                    }
                    MatchPrecision::Every => py == &rest,
                    MatchPrecision::Invalid => false,
                });
                if last {
                    let mut m = pre[j - 1].clone().unwrap();
                    m.push(i - 1);
                    return Some(m);
                }
            }
            // precision start
            if options.precision == MatchPrecision::Start {
                for py in muls {
                    let mut end = j;
                    let m = {
                        let mut m = pre[j - 1].clone().unwrap();
                        m.push(i - 1);
                        m
                    };
                    while end <= query.len()
                        && py.starts_with(&query[j - 1..end].iter().collect::<String>())
                    {
                        if current[end].as_ref().map(|v| v.len()).unwrap_or(0) < m.len() {
                            current[end] = Some(m.clone());
                        }
                        end += 1;
                    }
                }
            }
            // precision first
            if options.precision == MatchPrecision::First
                && muls.iter().any(|py| py.starts_with(query[j - 1]))
            {
                let mut m = pre[j - 1].clone().unwrap();
                m.push(i - 1);
                if current[j].as_ref().map(|v| v.len()).unwrap_or(0) < m.len() {
                    current[j] = Some(m);
                }
            }
            // complete pinyin match
            let rest: String = query[j - 1..].iter().collect();
            if let Some(found) = muls.iter().find(|py| rest.starts_with(py.as_str())) {
                let end_index = j - 1 + found.chars().count();
                let mut m = pre[j - 1].clone().unwrap();
                m.push(i - 1);
                if current[end_index].as_ref().map(|v| v.len()).unwrap_or(0) < m.len() {
                    current[end_index] = Some(m);
                }
            }
        }
        pre = current;
    }
    None
}
