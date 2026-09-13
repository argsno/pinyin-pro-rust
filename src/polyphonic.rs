//! `polyphonic()` ported from `lib/core/polyphonic/`.

use crate::options::{PolyInfo, PolyphonicOptions, PolyphonicOutput};
use crate::pinyin::{
    compile_scope, middleware_non_zh, middleware_pattern, middleware_tone_type, middleware_v, Sword,
};
use crate::pinyin_utils::{final_parts, first_letter, initial_and_final, num_of_tone};
use crate::store;

// Reuse the generic middlewares through a PinyinOptions view: polyphonic
// items go through pattern/toneType/v exactly like pinyin items.
fn apply_pattern(items: &mut [Sword], opts: &PolyphonicOptions) {
    let view = crate::options::PinyinOptions {
        pattern: opts.pattern,
        tone_type: opts.tone_type,
        initial_pattern: opts.initial_pattern,
        ..Default::default()
    };
    middleware_pattern(items, &view);
    middleware_tone_type(items, &view);
    middleware_v(items, &opts.v);
}

fn poly_list(text: &[char]) -> Vec<Sword> {
    text.iter()
        .map(|&c| {
            let ch = c.to_string();
            let p = store::custom_polyphonic()
                .read()
                .unwrap()
                .get_truthy(&ch)
                .or_else(|| store::dict1().read().unwrap().get_truthy(&ch));
            match p {
                Some(p) => Sword::new(ch, p, true),
                None => Sword::new(ch.clone(), ch, false),
            }
        })
        .collect()
}

fn split_polyphonic(list: Vec<Sword>) -> Vec<Vec<Sword>> {
    list.into_iter()
        .map(|item| {
            if !item.is_zh {
                return vec![item];
            }
            if !item.result.contains(' ') {
                return vec![Sword::new(item.origin.clone(), item.result.clone(), true)];
            }
            item.result
                .split(' ')
                .map(|p| Sword::new(item.origin.clone(), p.to_string(), true))
                .collect()
        })
        .collect()
}

/// Mirrors `polyphonic(text, options)`.
pub fn polyphonic(text: &str, options: PolyphonicOptions) -> PolyphonicOutput {
    let opts = options.normalized();
    if text.is_empty() {
        return PolyphonicOutput::Arr(Vec::new());
    }
    let chars: Vec<char> = text.chars().collect();
    let scope = compile_scope(&opts.non_zh_scope);
    let list = poly_list(&chars);
    let list = middleware_non_zh(list, opts.non_zh, &scope);
    let mut double = split_polyphonic(list);
    for item_list in double.iter_mut() {
        apply_pattern(item_list, &opts);
    }
    let is_all = opts.type_mode == crate::options::TypeMode::All;
    let is_array = opts.type_mode == crate::options::TypeMode::Array;
    if is_all {
        let all = double
            .into_iter()
            .map(|items| {
                items
                    .into_iter()
                    .map(|item| {
                        let pinyin = if item.is_zh {
                            item.result.clone()
                        } else {
                            String::new()
                        };
                        let (initial, fin) = initial_and_final(&pinyin, opts.initial_pattern);
                        let (head, body, tail) = final_parts(&pinyin);
                        PolyInfo {
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
                            in_zh_range: store::dict1()
                                .read()
                                .unwrap()
                                .get_truthy(&item.origin)
                                .is_some(),
                        }
                    })
                    .collect()
            })
            .collect();
        PolyphonicOutput::All(all)
    } else if is_array {
        PolyphonicOutput::Arr(
            double
                .into_iter()
                .map(|items| {
                    let mut seen = Vec::new();
                    for item in items {
                        if !seen.contains(&item.result) {
                            seen.push(item.result);
                        }
                    }
                    seen
                })
                .collect(),
        )
    } else {
        PolyphonicOutput::Str(
            double
                .into_iter()
                .map(|items| {
                    let mut seen = Vec::new();
                    for item in items {
                        if !seen.contains(&item.result) {
                            seen.push(item.result);
                        }
                    }
                    seen.join(" ")
                })
                .collect(),
        )
    }
}
