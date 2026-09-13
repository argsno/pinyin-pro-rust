//! Differential-test runner: reads ops JSON (argv[1]), executes them in
//! order against the Rust implementation, writes results JSON (argv[2]).
//!
//! Mirrors `run_js.mjs` op-for-op; outputs must be byte-identical to JS.

use std::env;
use std::fs;

use serde_json::{json, Value};

use pinyin_pro::options::*;
use pinyin_pro::segmentit::TokenizationAlgorithm;

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|x| x.to_string())
}

fn b(v: &Value, k: &str, d: bool) -> bool {
    v.get(k).and_then(|x| x.as_bool()).unwrap_or(d)
}

fn parse_v(v: &Value) -> VMode {
    match v.get("v") {
        Some(Value::Bool(true)) => VMode::V,
        Some(Value::Bool(false)) | None => VMode::Off,
        Some(Value::String(x)) => VMode::Custom(x.clone()),
        _ => VMode::Off,
    }
}

fn parse_segmentit(v: &Value) -> TokenizationAlgorithm {
    match v.get("segmentit").and_then(|x| x.as_u64()) {
        Some(1) => TokenizationAlgorithm::ReverseMaxMatch,
        Some(3) => TokenizationAlgorithm::MinTokenization,
        _ => TokenizationAlgorithm::MaxProbability,
    }
}

fn parse_surname_opt(v: &Value) -> Option<SurnameMode> {
    match v.get("surname").and_then(|x| x.as_str()) {
        Some("all") => Some(SurnameMode::All),
        Some("head") => Some(SurnameMode::Head),
        Some("off") => Some(SurnameMode::Off),
        _ => None,
    }
}

fn parse_mode(v: &Value) -> PinyinMode {
    match v.get("mode").and_then(|x| x.as_str()) {
        Some("surname") => PinyinMode::Surname,
        _ => PinyinMode::Normal,
    }
}

fn parse_pattern(v: &Value) -> PatternKind {
    match v.get("pattern").and_then(|x| x.as_str()) {
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

fn parse_tone(v: &Value) -> ToneType {
    match v.get("toneType").and_then(|x| x.as_str()) {
        Some("num") => ToneType::Num,
        Some("none") => ToneType::None,
        _ => ToneType::Symbol,
    }
}

fn parse_nonzh(v: &Value) -> NonZh {
    match v.get("nonZh").and_then(|x| x.as_str()) {
        Some("consecutive") => NonZh::Consecutive,
        Some("removed") => NonZh::Removed,
        _ => NonZh::Spaced,
    }
}

fn parse_pinyin_opts(o: &Value) -> PinyinOptions {
    PinyinOptions {
        pattern: parse_pattern(o),
        tone_type: parse_tone(o),
        type_mode: match o.get("type").and_then(|x| x.as_str()) {
            Some("array") => TypeMode::Array,
            Some("all") => TypeMode::All,
            _ => TypeMode::Str,
        },
        multiple: b(o, "multiple", false),
        mode: parse_mode(o),
        surname: parse_surname_opt(o),
        tone_sandhi: b(o, "toneSandhi", true),
        segmentit: parse_segmentit(o),
        non_zh: parse_nonzh(o),
        non_zh_scope: s(o, "nonZhScope"),
        remove_non_zh: b(o, "removeNonZh", false),
        v: parse_v(o),
        separator: s(o, "separator").unwrap_or_else(|| " ".to_string()),
        initial_pattern: match o.get("initialPattern").and_then(|x| x.as_str()) {
            Some("standard") => InitialPattern::Standard,
            _ => InitialPattern::Yw,
        },
        traditional: b(o, "traditional", false),
    }
}

fn all_info_json(i: &AllInfo) -> Value {
    json!({
      "origin": i.origin, "pinyin": i.pinyin, "initial": i.initial,
      "final": i.final_, "num": i.num, "first": i.first,
      "finalHead": i.final_head, "finalBody": i.final_body,
      "finalTail": i.final_tail, "isZh": i.is_zh,
      "polyphonic": i.polyphonic, "inZhRange": i.in_zh_range,
      "result": i.result,
    })
}

fn run_pinyin(o: &Value) -> Value {
    let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    match pinyin_pro::pinyin(text, parse_pinyin_opts(&opts)) {
        PinyinOutput::Str(x) => json!(x),
        PinyinOutput::Arr(x) => json!(x),
        PinyinOutput::All(x) => json!(x.iter().map(all_info_json).collect::<Vec<_>>()),
    }
}

fn run_polyphonic(o: &Value) -> Value {
    let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    let po = PolyphonicOptions {
        pattern: parse_pattern(&opts),
        tone_type: parse_tone(&opts),
        type_mode: match opts.get("type").and_then(|x| x.as_str()) {
            Some("array") => TypeMode::Array,
            Some("all") => TypeMode::All,
            _ => TypeMode::Str,
        },
        non_zh: parse_nonzh(&opts),
        non_zh_scope: s(&opts, "nonZhScope"),
        remove_non_zh: b(&opts, "removeNonZh", false),
        v: parse_v(&opts),
        initial_pattern: match opts.get("initialPattern").and_then(|x| x.as_str()) {
            Some("standard") => InitialPattern::Standard,
            _ => InitialPattern::Yw,
        },
    };
    match pinyin_pro::polyphonic(text, po) {
        PolyphonicOutput::Str(x) => json!(x),
        PolyphonicOutput::Arr(x) => json!(x),
        PolyphonicOutput::All(x) => json!(x
            .iter()
            .map(|g| g
                .iter()
                .map(|i| json!({
                  "origin": i.origin, "pinyin": i.pinyin, "initial": i.initial,
                  "final": i.final_, "num": i.num, "first": i.first,
                  "finalHead": i.final_head, "finalBody": i.final_body,
                  "finalTail": i.final_tail, "isZh": i.is_zh,
                  "inZhRange": i.in_zh_range,
                }))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()),
    }
}

fn run_match(o: &Value) -> Value {
    let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
    let query = o.get("query").and_then(|x| x.as_str()).unwrap_or("");
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    let mo = MatchOptions {
        precision: match opts.get("precision").and_then(|x| x.as_str()) {
            None => MatchPrecision::First,
            Some("first") => MatchPrecision::First,
            Some("start") => MatchPrecision::Start,
            Some("every") => MatchPrecision::Every,
            Some("any") => MatchPrecision::Any,
            _ => MatchPrecision::Invalid,
        },
        continuous: b(&opts, "continuous", false),
        space: match opts.get("space").and_then(|x| x.as_str()) {
            None | Some("ignore") => MatchSpace::Ignore,
            _ => MatchSpace::Preserve,
        },
        last_precision: match opts.get("lastPrecision").and_then(|x| x.as_str()) {
            None => MatchPrecision::Start,
            Some("any") => MatchPrecision::Any,
            Some("every") => MatchPrecision::Every,
            Some("first") => MatchPrecision::First,
            Some("start") => MatchPrecision::Start,
            _ => MatchPrecision::Invalid,
        },
        insensitive: b(&opts, "insensitive", true),
        v: parse_v(&opts),
    };
    match pinyin_pro::match_pinyin(text, query, mo) {
        Some(v) => json!(v),
        None => Value::Null,
    }
}

fn run_convert(o: &Value) -> Value {
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    let co = ConvertOptions {
        separator: s(&opts, "separator").unwrap_or_else(|| " ".to_string()),
        format: match opts.get("format").and_then(|x| x.as_str()) {
            Some("symbolToNum") => ConvertFormat::SymbolToNum,
            Some("toneNone") => ConvertFormat::ToneNone,
            _ => ConvertFormat::NumToSymbol,
        },
    };
    if let Some(items) = o.get("items").and_then(|x| x.as_array()) {
        let items: Vec<String> = items
            .iter()
            .filter_map(|x| x.as_str().map(|x| x.to_string()))
            .collect();
        json!(pinyin_pro::convert_array(items, co))
    } else {
        let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
        json!(pinyin_pro::convert(text, co))
    }
}

fn run_html(o: &Value) -> Value {
    let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    let mut custom_class_map = Vec::new();
    if let Some(m) = opts.get("customClassMap").and_then(|x| x.as_object()) {
        for (k, v) in m {
            if let Some(arr) = v.as_array() {
                custom_class_map.push((
                    k.clone(),
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|x| x.to_string()))
                        .collect(),
                ));
            }
        }
    }
    let ho = HtmlOptions {
        tone_type: parse_tone(&opts),
        v: parse_v(&opts),
        tone_sandhi: b(&opts, "toneSandhi", true),
        segmentit: parse_segmentit(&opts),
        traditional: b(&opts, "traditional", false),
        surname: parse_surname_opt(&opts),
        mode: parse_mode(&opts),
        result_class: s(&opts, "resultClass").unwrap_or_else(|| "py-result-item".to_string()),
        pinyin_class: s(&opts, "pinyinClass").unwrap_or_else(|| "py-pinyin-item".to_string()),
        chinese_class: s(&opts, "chineseClass").unwrap_or_else(|| "py-chinese-item".to_string()),
        wrap_non_chinese: b(&opts, "wrapNonChinese", false),
        non_chinese_class: s(&opts, "nonChineseClass")
            .unwrap_or_else(|| "py-non-chinese-item".to_string()),
        custom_class_map,
        rp: b(&opts, "rp", true),
    };
    json!(pinyin_pro::html(text, ho))
}

fn run_segment(o: &Value) -> Value {
    let text = o.get("text").and_then(|x| x.as_str()).unwrap_or("");
    let opts = o.get("options").cloned().unwrap_or(Value::Null);
    let format = match opts.get("format").and_then(|x| x.as_u64()) {
        Some(2) => OutputFormat::AllArray,
        Some(3) => OutputFormat::AllString,
        Some(4) => OutputFormat::PinyinSegment,
        Some(5) => OutputFormat::PinyinArray,
        Some(6) => OutputFormat::PinyinString,
        Some(7) => OutputFormat::ZhSegment,
        Some(8) => OutputFormat::ZhArray,
        Some(9) => OutputFormat::ZhString,
        _ => OutputFormat::AllSegment,
    };
    let so = SegmentOptions {
        tone_type: parse_tone(&opts),
        mode: parse_mode(&opts),
        surname: parse_surname_opt(&opts),
        non_zh: parse_nonzh(&opts),
        non_zh_scope: s(&opts, "nonZhScope"),
        v: parse_v(&opts),
        tone_sandhi: b(&opts, "toneSandhi", true),
        segmentit: parse_segmentit(&opts),
        traditional: b(&opts, "traditional", false),
        format,
        separator: s(&opts, "separator").unwrap_or_else(|| " ".to_string()),
    };
    let pair = |p: &SegmentPair| json!({"origin": p.origin, "result": p.result});
    match pinyin_pro::segment(text, so) {
        SegmentOutput::AllSegment(v) => json!(v.iter().map(pair).collect::<Vec<_>>()),
        SegmentOutput::AllArray(v) => json!(v
            .iter()
            .map(|g| g.iter().map(pair).collect::<Vec<_>>())
            .collect::<Vec<_>>()),
        SegmentOutput::AllString(p) => pair(&p),
        SegmentOutput::PinyinSegment(v) => json!(v),
        SegmentOutput::PinyinArray(v) => json!(v),
        SegmentOutput::PinyinString(x) => json!(x),
        SegmentOutput::ZhSegment(v) => json!(v),
        SegmentOutput::ZhArray(v) => json!(v),
        SegmentOutput::ZhString(x) => json!(x),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let ops: Vec<Value> = serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
    let mut out = Vec::with_capacity(ops.len());
    for o in &ops {
        let op = o.get("op").and_then(|x| x.as_str()).unwrap_or("");
        out.push(match op {
            "pinyin" => run_pinyin(o),
            "polyphonic" => run_polyphonic(o),
            "match" => run_match(o),
            "convert" => run_convert(o),
            "convert_arr" => run_convert(o),
            "html" => run_html(o),
            "segment" => run_segment(o),
            "customPinyin" => {
                let empty = serde_json::Map::new();
                let cfg = o
                    .get("config")
                    .and_then(|x| x.as_object())
                    .unwrap_or(&empty);
                let config: Vec<(String, String)> = cfg
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|x| (k.clone(), x.to_string())))
                    .collect();
                let co = o.get("options").cloned().unwrap_or(Value::Null);
                let h = |k: &str| match co.get(k).and_then(|x| x.as_str()) {
                    Some("replace") => Some(pinyin_pro::custom::CustomHandle::Replace),
                    Some("add") => Some(pinyin_pro::custom::CustomHandle::Add),
                    _ => None,
                };
                pinyin_pro::custom_pinyin(
                    &config,
                    pinyin_pro::custom::CustomPinyinOptions {
                        multiple: h("multiple"),
                        polyphonic: h("polyphonic"),
                    },
                );
                json!("ok")
            }
            "clearCustomDict" => {
                use pinyin_pro::custom::CustomDictKind::*;
                let kinds: Vec<pinyin_pro::custom::CustomDictKind> = match o.get("kinds") {
                    Some(Value::String(x)) => vec![match x.as_str() {
                        "multiple" => Multiple,
                        "polyphonic" => Polyphonic,
                        _ => Pinyin,
                    }],
                    Some(Value::Array(a)) => a
                        .iter()
                        .filter_map(|x| {
                            x.as_str().map(|x| match x {
                                "multiple" => Multiple,
                                "polyphonic" => Polyphonic,
                                _ => Pinyin,
                            })
                        })
                        .collect(),
                    _ => vec![Pinyin, Multiple, Polyphonic],
                };
                pinyin_pro::clear_custom_dict(&kinds);
                json!("ok")
            }
            "addDict" => {
                let dict: Vec<(String, pinyin_pro::dict_api::DictValue)> = o
                    .get("dict")
                    .and_then(|x| x.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|e| {
                                let k = e.get(0)?.as_str()?.to_string();
                                let vv = e.get(1)?;
                                let dv = match vv {
                                    Value::String(x) => {
                                        pinyin_pro::dict_api::DictValue::from(x.as_str())
                                    }
                                    Value::Array(a) => pinyin_pro::dict_api::DictValue {
                                        pinyin: a.first()?.as_str()?.to_string(),
                                        probability: a.get(1)?.as_f64(),
                                        pos: a
                                            .get(2)
                                            .and_then(|x| x.as_str())
                                            .map(|x| x.to_string()),
                                        is_array: true,
                                    },
                                    _ => return None,
                                };
                                Some((k, dv))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let oo = o.get("options").cloned().unwrap_or(Value::Null);
                let name = if oo.is_string() {
                    oo.as_str().map(|x| x.to_string())
                } else {
                    s(&oo, "name")
                };
                let d1 = match oo.get("dict1").and_then(|x| x.as_str()) {
                    Some("replace") => pinyin_pro::dict_api::Dict1Handle::Replace,
                    Some("ignore") => pinyin_pro::dict_api::Dict1Handle::Ignore,
                    _ => pinyin_pro::dict_api::Dict1Handle::Add,
                };
                pinyin_pro::add_dict(
                    &dict,
                    pinyin_pro::dict_api::AddDictOptions { name, dict1: d1 },
                );
                json!("ok")
            }
            "removeDict" => {
                let name = o.get("name").and_then(|x| x.as_str());
                pinyin_pro::remove_dict(name);
                json!("ok")
            }
            "addTraditionalDict" => {
                let empty = serde_json::Map::new();
                let d = o.get("dict").and_then(|x| x.as_object()).unwrap_or(&empty);
                let dict: Vec<(String, String)> = d
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|x| (k.clone(), x.to_string())))
                    .collect();
                pinyin_pro::add_traditional_dict(&dict);
                json!("ok")
            }
            "getInitialAndFinal" => {
                let p = o.get("pinyin").and_then(|x| x.as_str()).unwrap_or("");
                let ip = match o.get("initialPattern").and_then(|x| x.as_str()) {
                    Some("standard") => InitialPattern::Standard,
                    _ => InitialPattern::Yw,
                };
                let (initial, fin) = pinyin_pro::initial_and_final(p, ip);
                json!({"initial": initial, "final": fin})
            }
            "getFinalParts" => {
                let p = o.get("pinyin").and_then(|x| x.as_str()).unwrap_or("");
                let (head, body, tail) = pinyin_pro::final_parts(p);
                json!({"head": head, "body": body, "tail": tail})
            }
            "getNumOfTone" => {
                let p = o.get("pinyin").and_then(|x| x.as_str()).unwrap_or("");
                json!(pinyin_pro::num_of_tone(p))
            }
            _ => json!({"error": "unknown op"}),
        });
    }
    fs::write(&args[2], serde_json::to_string(&json!(out)).unwrap()).unwrap();
    eprintln!("rust done: {} ops", out.len());
}
