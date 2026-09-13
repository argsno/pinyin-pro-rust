//! Pure pinyin string helpers ported from `lib/core/pinyin/handle.ts`.

use crate::data::special::{
    DOUBLE_FINAL_LIST, INITIAL_LIST, SPECIAL_FINAL_LIST, SPECIAL_FINAL_MAP, SPECIAL_INITIAL_LIST,
};
use crate::options::InitialPattern;

// Combining marks used after n/m/ê in the data (decomposed toned forms).
const COMBINING_MACRON: char = '\u{304}';
const COMBINING_GRAVE: char = '\u{300}';
const COMBINING_CARON: char = '\u{30c}';

// Literal decomposed sequences mirrored from the JS regexes.
const N_MACRON: &str = "n\u{304}";
const M_MACRON: &str = "m\u{304}";
const M_CARON: &str = "m\u{30c}";
const M_GRAVE: &str = "m\u{300}";
const E_CIRC_MACRON: &str = "ê\u{304}";
const E_CIRC_CARON: &str = "ê\u{30c}";

/// Map a precomposed toned char to its toneless base (single-char cases).
fn toneless_base(c: char) -> Option<char> {
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

/// `getPinyinWithoutTone`: strip tone marks, keeping the exact JS semantics
/// (precomposed map + the n/m/ê + combining-mark sequences).
pub fn strip_tone(pinyin: &str) -> String {
    let chars: Vec<char> = pinyin.chars().collect();
    let mut out = String::with_capacity(pinyin.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if i + 1 < chars.len() {
            let n = chars[i + 1];
            let decomposed = (c == 'n' || c == 'm') && n == COMBINING_MACRON
                || c == 'm' && (n == COMBINING_CARON || n == COMBINING_GRAVE)
                || c == 'ê' && (n == COMBINING_MACRON || n == COMBINING_CARON);
            if decomposed {
                out.push(c);
                i += 2;
                continue;
            }
        }
        match toneless_base(c) {
            Some(b) => out.push(b),
            None => out.push(c),
        }
        i += 1;
    }
    out
}

fn has_tone1(s: &str) -> bool {
    s.contains('ā')
        || s.contains('ō')
        || s.contains('ē')
        || s.contains('ī')
        || s.contains('ū')
        || s.contains('ǖ')
        || s.contains(N_MACRON)
        || s.contains(M_MACRON)
        || s.contains(E_CIRC_MACRON)
}

fn has_tone2(s: &str) -> bool {
    s.contains('á')
        || s.contains('ó')
        || s.contains('é')
        || s.contains('í')
        || s.contains('ú')
        || s.contains('ǘ')
        || s.contains('ń')
        || s.contains('ḿ')
        || s.contains('ế')
}

fn has_tone3(s: &str) -> bool {
    s.contains('ǎ')
        || s.contains('ǒ')
        || s.contains('ě')
        || s.contains('ǐ')
        || s.contains('ǔ')
        || s.contains('ǚ')
        || s.contains('ň')
        || s.contains(M_CARON)
        || s.contains(E_CIRC_CARON)
}

fn has_tone4(s: &str) -> bool {
    s.contains('à')
        || s.contains('ò')
        || s.contains('è')
        || s.contains('ì')
        || s.contains('ù')
        || s.contains('ǜ')
        || s.contains('ǹ')
        || s.contains(M_GRAVE)
        || s.contains('ề')
}

fn has_plain_vowel(s: &str) -> bool {
    s.contains('a')
        || s.contains('o')
        || s.contains('e')
        || s.contains('i')
        || s.contains('u')
        || s.contains('ü')
        || s.contains('ê')
}

/// `getNumOfTone`: toned pinyin -> space-separated tone digits.
pub fn num_of_tone(pinyin: &str) -> String {
    pinyin
        .split(' ')
        .map(|tok| {
            if has_tone1(tok) {
                "1"
            } else if has_tone2(tok) {
                "2"
            } else if has_tone3(tok) {
                "3"
            } else if has_tone4(tok) {
                "4"
            } else if has_plain_vowel(tok) || tok.ends_with('n') || tok.ends_with('m') {
                "0"
            } else {
                ""
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `getPinyinWithNum`: toned pinyin + origin -> `pin1`-style pinyin.
pub fn pinyin_with_num(pinyin: &str, origin_pinyin: &str) -> String {
    let stripped_owned = strip_tone(pinyin);
    let tones_owned = num_of_tone(origin_pinyin);
    let tones: Vec<&str> = tones_owned.split(' ').collect();
    stripped_owned
        .split(' ')
        .enumerate()
        .map(|(i, item)| {
            let t = tones.get(i).copied().unwrap_or("");
            format!("{item}{t}")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `getFirstLetter`.
pub fn first_letter(pinyin: &str, is_zh: bool) -> String {
    pinyin
        .split(' ')
        .map(|tok| {
            if is_zh {
                tok.chars()
                    .next()
                    .map(|c| c.to_string())
                    .unwrap_or_default()
            } else {
                tok.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `getInitialAndFinal`.
pub fn initial_and_final(pinyin: &str, initial_pattern: InitialPattern) -> (String, String) {
    let mut initials = Vec::new();
    let mut finals = Vec::new();
    for tok in pinyin.split(' ') {
        for init in INITIAL_LIST {
            if let Some(fin) = tok.strip_prefix(init) {
                let mut fin = fin.to_string();
                if SPECIAL_INITIAL_LIST.contains(init) && SPECIAL_FINAL_LIST.contains(&fin.as_str())
                {
                    if let Some(mapped) = SPECIAL_FINAL_MAP.iter().find(|(k, _)| *k == fin) {
                        fin = mapped.1.to_string();
                    }
                }
                initials.push(init.to_string());
                finals.push(fin);
                break;
            }
        }
    }
    if initial_pattern == InitialPattern::Standard {
        for init in initials.iter_mut() {
            if init == "y" || init == "w" {
                *init = String::new();
            }
        }
    }
    (initials.join(" "), finals.join(" "))
}

/// `getFinalPartsFromFinal`.
pub fn final_parts_from_final(fin: &str) -> (String, String, String) {
    if DOUBLE_FINAL_LIST.contains(&strip_tone(fin).as_str()) {
        let chars: Vec<char> = fin.chars().collect();
        let head = chars.first().map(|c| c.to_string()).unwrap_or_default();
        let body = chars.get(1).map(|c| c.to_string()).unwrap_or_default();
        let tail: String = chars.iter().skip(2).collect();
        (head, body, tail)
    } else {
        let chars: Vec<char> = fin.chars().collect();
        let body = chars.first().map(|c| c.to_string()).unwrap_or_default();
        let tail: String = chars.iter().skip(1).collect();
        (String::new(), body, tail)
    }
}

/// `getFinalParts`.
pub fn final_parts(pinyin: &str) -> (String, String, String) {
    let (_, fin) = initial_and_final(pinyin, InitialPattern::Yw);
    final_parts_from_final(&fin)
}
