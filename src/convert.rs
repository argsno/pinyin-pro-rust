//! `convert()` ported from `lib/core/convert/index.ts`.

use crate::options::{ConvertFormat, ConvertOptions};
use crate::pinyin_utils::{num_of_tone, strip_tone};

/// (plain, [toneless, tone1..tone4]) in JS object iteration order.
static TONE_MAP: &[(&str, [&str; 5])] = &[
    ("a", ["a", "ā", "á", "ǎ", "à"]),
    ("o", ["o", "ō", "ó", "ǒ", "ò"]),
    ("e", ["e", "ē", "é", "ě", "è"]),
    ("ü", ["ü", "ǖ", "ǘ", "ǚ", "ǜ"]),
    ("v", ["ü", "ǖ", "ǘ", "ǚ", "ǜ"]),
    ("ui", ["ui", "uī", "uí", "uǐ", "uì"]),
    ("iu", ["iu", "iū", "iú", "iǔ", "iù"]),
    ("i", ["i", "ī", "í", "ǐ", "ì"]),
    ("u", ["u", "ū", "ú", "ǔ", "ù"]),
    ("n", ["n", "n̄", "ń", "ň", "ǹ"]),
    ("m", ["m", "m̄", "ḿ", "m̌", "m̀"]),
    ("ê", ["ê", "ê̄", "ế", "ê̌", "ề"]),
];

fn is_tone_digit(c: char) -> bool {
    matches!(c, '0' | '1' | '2' | '3' | '4')
}

fn format_num_to_symbol(pinyin: &str) -> String {
    let mut p = pinyin.to_string();
    let mut suffix = "";
    let chars: Vec<char> = p.chars().collect();
    if chars.len() > 2 && chars[chars.len() - 1] == 'r' && is_tone_digit(chars[chars.len() - 2]) {
        suffix = "r";
        p = chars[..chars.len() - 1].iter().collect();
    }
    let chars: Vec<char> = p.chars().collect();
    if chars.last().map(|c| is_tone_digit(*c)).unwrap_or(false) {
        let tone = chars[chars.len() - 1].to_digit(10).unwrap() as usize;
        for (key, forms) in TONE_MAP {
            if p.contains(key) {
                let body: String = chars[..chars.len() - 1].iter().collect();
                // JS String.replace with a string pattern replaces the first hit.
                return format!("{}{}", body.replacen(key, forms[tone], 1), suffix);
            }
        }
        return format!("{p}{suffix}");
    }
    format!("{p}{suffix}")
}

fn is_single_e_r(pinyin: &str) -> bool {
    let chars: Vec<char> = pinyin.chars().collect();
    chars.len() == 2 && matches!(chars[0], 'e' | 'ē' | 'é' | 'ě' | 'è') && chars[1] == 'r'
}

fn format_symbol_to_num(pinyin: &str) -> String {
    let chars: Vec<char> = pinyin.chars().collect();
    if chars.len() > 1 && chars[chars.len() - 1] == 'r' && !is_single_e_r(pinyin) {
        let without_r: String = chars[..chars.len() - 1].iter().collect();
        let tone = num_of_tone(&without_r);
        if tone != "0" && !tone.is_empty() {
            return format!("{}{}r", strip_tone(&without_r), tone);
        }
    }
    format!("{}{}", strip_tone(pinyin), num_of_tone(pinyin))
}

/// Mirrors `convert(pinyin, options)` for a pre-split list.
pub fn convert_list(items: Vec<String>, options: &ConvertOptions) -> Vec<String> {
    items
        .into_iter()
        .map(|item| match options.format {
            ConvertFormat::NumToSymbol => format_num_to_symbol(&item),
            ConvertFormat::SymbolToNum => format_symbol_to_num(&item),
            ConvertFormat::ToneNone => strip_tone(&item),
        })
        .collect()
}

/// Mirrors `convert(pinyin, options)`.
pub fn convert(text: &str, options: ConvertOptions) -> String {
    let items: Vec<String> = if options.separator.is_empty() {
        // JS 'abc'.split('') -> chars
        text.chars().map(|c| c.to_string()).collect()
    } else {
        text.split(&options.separator)
            .map(|s| s.to_string())
            .collect()
    };
    convert_list(items, &options).join(&options.separator)
}

/// Array variant of `convert`.
pub fn convert_array(items: Vec<String>, options: ConvertOptions) -> Vec<String> {
    convert_list(items, &options)
}
