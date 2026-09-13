//! Node.js native bindings (feature `napi`, via napi-rs).
//!
//! The surface mirrors the JS package but crosses the boundary as JSON:
//! options go in as a JSON string, results come out as JSON strings.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::Deserialize;

use crate::options::*;
use crate::segmentit::TokenizationAlgorithm;

#[derive(Deserialize, Default)]
#[serde(default)]
struct PinyinArgs {
    pattern: Option<String>,
    tone_type: Option<String>,
    #[serde(rename = "type")]
    type_mode: Option<String>,
    multiple: Option<bool>,
    mode: Option<String>,
    surname: Option<String>,
    tone_sandhi: Option<bool>,
    segmentit: Option<u8>,
    non_zh: Option<String>,
    remove_non_zh: Option<bool>,
    v: Option<serde_json::Value>,
    separator: Option<String>,
    initial_pattern: Option<String>,
    traditional: Option<bool>,
}

fn parse_pattern(s: Option<String>) -> PatternKind {
    match s.as_deref() {
        Some("initial") => PatternKind::Initial,
        Some("final") => PatternKind::Final,
        Some("num") => PatternKind::Num,
        Some("first") => PatternKind::First,
        Some("finalHead") => PatternKind::FinalHead,
        Some("finalBody") => PatternKind::FinalBody,
        Some("finalTail") => PatternKind::FinalTail,
        _ => PatternKind::Pinyin,
    }
}

fn parse_tone(s: Option<String>) -> ToneType {
    match s.as_deref() {
        Some("num") => ToneType::Num,
        Some("none") => ToneType::None,
        _ => ToneType::Symbol,
    }
}

fn parse_v(v: Option<serde_json::Value>) -> VMode {
    match v {
        Some(serde_json::Value::Bool(true)) => VMode::V,
        Some(serde_json::Value::String(s)) => VMode::Custom(s),
        _ => VMode::Off,
    }
}

fn parse_opts(a: PinyinArgs) -> PinyinOptions {
    PinyinOptions {
        pattern: parse_pattern(a.pattern),
        tone_type: parse_tone(a.tone_type),
        type_mode: match a.type_mode.as_deref() {
            Some("array") => TypeMode::Array,
            Some("all") => TypeMode::All,
            _ => TypeMode::Str,
        },
        multiple: a.multiple.unwrap_or(false),
        mode: match a.mode.as_deref() {
            Some("surname") => PinyinMode::Surname,
            _ => PinyinMode::Normal,
        },
        surname: match a.surname.as_deref() {
            Some("all") => Some(SurnameMode::All),
            Some("head") => Some(SurnameMode::Head),
            Some("off") => Some(SurnameMode::Off),
            _ => None,
        },
        tone_sandhi: a.tone_sandhi.unwrap_or(true),
        segmentit: match a.segmentit {
            Some(1) => TokenizationAlgorithm::ReverseMaxMatch,
            Some(3) => TokenizationAlgorithm::MinTokenization,
            _ => TokenizationAlgorithm::MaxProbability,
        },
        non_zh: match a.non_zh.as_deref() {
            Some("consecutive") => NonZh::Consecutive,
            Some("removed") => NonZh::Removed,
            _ => NonZh::Spaced,
        },
        non_zh_scope: None,
        remove_non_zh: a.remove_non_zh.unwrap_or(false),
        v: parse_v(a.v),
        separator: a.separator.unwrap_or_else(|| " ".to_string()),
        initial_pattern: match a.initial_pattern.as_deref() {
            Some("standard") => InitialPattern::Standard,
            _ => InitialPattern::Yw,
        },
        traditional: a.traditional.unwrap_or(false),
    }
}

/// Convert Chinese text to pinyin. Returns a JSON string:
/// a string for `type: string`, an array for `array`, objects for `all`.
#[napi]
pub fn pinyin(text: String, options_json: Option<String>) -> Result<String> {
    let args: PinyinArgs = options_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| Error::from_reason(format!("invalid options JSON: {e}")))?
        .unwrap_or_default();
  let out = crate::pinyin::pinyin(&text, parse_opts(args));
  serde_json::to_string(&PinyinJson::from(out))
    .map_err(|e| Error::from_reason(e.to_string()))
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum PinyinJson {
    Str(String),
    Arr(Vec<String>),
    All(Vec<AllInfoJson>),
}

#[derive(serde::Serialize)]
struct AllInfoJson {
    origin: String,
    pinyin: String,
    initial: String,
    #[serde(rename = "final")]
    final_: String,
    num: i64,
    first: String,
    #[serde(rename = "finalHead")]
    final_head: String,
    #[serde(rename = "finalBody")]
    final_body: String,
    #[serde(rename = "finalTail")]
    final_tail: String,
    #[serde(rename = "isZh")]
    is_zh: bool,
    polyphonic: Vec<String>,
    #[serde(rename = "inZhRange")]
    in_zh_range: bool,
    result: String,
}

impl From<PinyinOutput> for PinyinJson {
    fn from(o: PinyinOutput) -> Self {
        match o {
            PinyinOutput::Str(s) => PinyinJson::Str(s),
            PinyinOutput::Arr(a) => PinyinJson::Arr(a),
            PinyinOutput::All(all) => PinyinJson::All(
                all.into_iter()
                    .map(|i| AllInfoJson {
                        origin: i.origin,
                        pinyin: i.pinyin,
                        initial: i.initial,
                        final_: i.final_,
                        num: i.num,
                        first: i.first,
                        final_head: i.final_head,
                        final_body: i.final_body,
                        final_tail: i.final_tail,
                        is_zh: i.is_zh,
                        polyphonic: i.polyphonic,
                        in_zh_range: i.in_zh_range,
                        result: i.result,
                    })
                    .collect(),
            ),
        }
    }
}

/// Detect whether `text` matches `query` pinyin. Returns matched indices
/// as a JSON array string, or `null`.
#[napi]
pub fn match_text(
  text: String,
  query: String,
  options_json: Option<String>,
) -> Result<String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct M {
        precision: Option<String>,
        continuous: Option<bool>,
        space: Option<String>,
        insensitive: Option<bool>,
        last_precision: Option<String>,
    }
    let a: M = options_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| Error::from_reason(format!("invalid options JSON: {e}")))?
        .unwrap_or_default();
    let opts = MatchOptions {
        precision: match a.precision.as_deref() {
            None => MatchPrecision::First,
            Some("first") => MatchPrecision::First,
            Some("start") => MatchPrecision::Start,
            Some("every") => MatchPrecision::Every,
            Some("any") => MatchPrecision::Any,
            _ => MatchPrecision::Invalid,
        },
        continuous: a.continuous.unwrap_or(false),
        space: match a.space.as_deref() {
            None | Some("ignore") => MatchSpace::Ignore,
            _ => MatchSpace::Preserve,
        },
        last_precision: match a.last_precision.as_deref() {
            None => MatchPrecision::Start,
            Some("any") => MatchPrecision::Any,
            Some("every") => MatchPrecision::Every,
            Some("first") => MatchPrecision::First,
            Some("start") => MatchPrecision::Start,
            _ => MatchPrecision::Invalid,
        },
        insensitive: a.insensitive.unwrap_or(true),
        v: VMode::Off,
    };
  Ok(match crate::match_::match_text(&text, &query, opts) {
    Some(v) => serde_json::json!(v).to_string(),
    None => "null".to_string(),
  })
}

/// Convert between pinyin formats (`pin1` <-> `pīn`).
#[napi]
pub fn convert(text: String, options_json: Option<String>) -> Result<String> {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct C {
        separator: Option<String>,
        format: Option<String>,
    }
    let a: C = options_json
        .as_deref()
        .map(serde_json::from_str)
        .transpose()
        .map_err(|e| Error::from_reason(format!("invalid options JSON: {e}")))?
        .unwrap_or_default();
    let opts = ConvertOptions {
        separator: a.separator.unwrap_or_else(|| " ".to_string()),
        format: match a.format.as_deref() {
            Some("symbolToNum") => ConvertFormat::SymbolToNum,
            Some("toneNone") => ConvertFormat::ToneNone,
            _ => ConvertFormat::NumToSymbol,
        },
    };
    Ok(crate::convert::convert(&text, opts))
}
