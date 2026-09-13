//! WebAssembly bindings (feature `wasm`, via wasm-bindgen).
//!
//! JSON in / JSON out, mirroring the napi module surface.

use wasm_bindgen::prelude::*;

// Reuse the napi option parsing by delegating at the JSON level would
// couple the features; instead expose the same three core functions here
// with duplicated minimal parsing.

#[wasm_bindgen]
pub fn pinyin_wasm(text: &str, options_json: Option<String>) -> String {
    // Delegate to the same option shape as napi (duplicated to avoid a
    // cross-feature dependency).
    let v: serde_json::Value = options_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(serde_json::Value::Null);
    let get = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|s| s.to_string());
    let opts = crate::options::PinyinOptions {
        pattern: match get("pattern").as_deref() {
            Some("initial") => crate::options::PatternKind::Initial,
            Some("final") => crate::options::PatternKind::Final,
            Some("num") => crate::options::PatternKind::Num,
            Some("first") => crate::options::PatternKind::First,
            Some("finalHead") => crate::options::PatternKind::FinalHead,
            Some("finalBody") => crate::options::PatternKind::FinalBody,
            Some("finalTail") => crate::options::PatternKind::FinalTail,
            _ => crate::options::PatternKind::Pinyin,
        },
        tone_type: match get("toneType").as_deref() {
            Some("num") => crate::options::ToneType::Num,
            Some("none") => crate::options::ToneType::None,
            _ => crate::options::ToneType::Symbol,
        },
        type_mode: match get("type").as_deref() {
            Some("array") => crate::options::TypeMode::Array,
            Some("all") => crate::options::TypeMode::All,
            _ => crate::options::TypeMode::Str,
        },
        multiple: v.get("multiple").and_then(|x| x.as_bool()).unwrap_or(false),
        mode: match get("mode").as_deref() {
            Some("surname") => crate::options::PinyinMode::Surname,
            _ => crate::options::PinyinMode::Normal,
        },
        surname: match get("surname").as_deref() {
            Some("all") => Some(crate::options::SurnameMode::All),
            Some("head") => Some(crate::options::SurnameMode::Head),
            Some("off") => Some(crate::options::SurnameMode::Off),
            _ => None,
        },
        tone_sandhi: v
            .get("toneSandhi")
            .and_then(|x| x.as_bool())
            .unwrap_or(true),
        segmentit: match v.get("segmentit").and_then(|x| x.as_u64()) {
            Some(1) => crate::segmentit::TokenizationAlgorithm::ReverseMaxMatch,
            Some(3) => crate::segmentit::TokenizationAlgorithm::MinTokenization,
            _ => crate::segmentit::TokenizationAlgorithm::MaxProbability,
        },
        non_zh: match get("nonZh").as_deref() {
            Some("consecutive") => crate::options::NonZh::Consecutive,
            Some("removed") => crate::options::NonZh::Removed,
            _ => crate::options::NonZh::Spaced,
        },
        non_zh_scope: None,
        remove_non_zh: v
            .get("removeNonZh")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
        v: match v.get("v") {
            Some(serde_json::Value::Bool(true)) => crate::options::VMode::V,
            Some(serde_json::Value::String(s)) => crate::options::VMode::Custom(s.clone()),
            _ => crate::options::VMode::Off,
        },
        separator: get("separator").unwrap_or_else(|| " ".to_string()),
        initial_pattern: match get("initialPattern").as_deref() {
            Some("standard") => crate::options::InitialPattern::Standard,
            _ => crate::options::InitialPattern::Yw,
        },
        traditional: v
            .get("traditional")
            .and_then(|x| x.as_bool())
            .unwrap_or(false),
    };
    let out = crate::pinyin::pinyin(text, opts);
    match out {
        crate::options::PinyinOutput::Str(s) => serde_json::json!(s).to_string(),
        crate::options::PinyinOutput::Arr(a) => serde_json::json!(a).to_string(),
        crate::options::PinyinOutput::All(all) => serde_json::json!(all
            .iter()
            .map(|i| serde_json::json!({
              "origin": i.origin, "pinyin": i.pinyin,
              "initial": i.initial, "final": i.final_,
              "num": i.num, "first": i.first,
              "finalHead": i.final_head, "finalBody": i.final_body,
              "finalTail": i.final_tail, "isZh": i.is_zh,
              "polyphonic": i.polyphonic, "inZhRange": i.in_zh_range,
              "result": i.result,
            }))
            .collect::<Vec<_>>())
        .to_string(),
    }
}

#[wasm_bindgen]
pub fn match_wasm(text: &str, query: &str) -> String {
    let opts = crate::options::MatchOptions::default();
    match crate::match_::match_text(text, query, opts) {
        Some(v) => serde_json::json!(v).to_string(),
        None => "null".to_string(),
    }
}

#[wasm_bindgen]
pub fn convert_wasm(text: &str, format: Option<String>) -> String {
    let opts = crate::options::ConvertOptions {
        format: match format.as_deref() {
            Some("symbolToNum") => crate::options::ConvertFormat::SymbolToNum,
            Some("toneNone") => crate::options::ConvertFormat::ToneNone,
            _ => crate::options::ConvertFormat::NumToSymbol,
        },
        ..Default::default()
    };
    crate::convert::convert(text, opts)
}
