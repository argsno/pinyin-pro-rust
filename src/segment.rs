//! `segment()` ported from `lib/core/segment/`.

use crate::options::{
    OutputFormat, PinyinMode, PinyinOptions, SegmentOptions, SegmentOutput, SegmentPair,
    SurnameMode,
};
use crate::pinyin::{compile_scope, get_pinyin_with_algo, middleware_non_zh};
use crate::pinyin::{middleware_tone_sandhi, middleware_tone_type};
use crate::pinyin::{middleware_v, Sword};
use crate::segmentit::MatchPattern;

#[derive(Clone, Debug)]
struct OriginSegment {
    segment: Vec<SegmentPair>,
}

fn middleware_segment(list: &[Sword], matches: &[MatchPattern]) -> Vec<OriginSegment> {
    let mut segments = Vec::new();
    let mut i = 0;
    let mut j = 0;
    while i < list.len() && j < matches.len() {
        let m = &matches[j];
        let item = &list[i];
        if m.zh.starts_with(&item.origin) {
            let start = i;
            let chars: Vec<char> = m.zh.chars().collect();
            let mut cur = start + 1;
            while cur < list.len()
                && chars.get(cur - start).map(|c| c.to_string()).as_deref()
                    == Some(list[cur].origin.as_str())
            {
                cur += 1;
            }
            segments.push(OriginSegment {
                segment: list[start..cur]
                    .iter()
                    .map(|it| SegmentPair {
                        origin: it.origin.clone(),
                        result: it.result.clone(),
                    })
                    .collect(),
            });
            i = cur;
            j += 1;
        } else {
            segments.push(OriginSegment {
                segment: vec![SegmentPair {
                    origin: item.origin.clone(),
                    result: item.result.clone(),
                }],
            });
            i += 1;
        }
    }
    while i < list.len() {
        segments.push(OriginSegment {
            segment: vec![SegmentPair {
                origin: list[i].origin.clone(),
                result: list[i].result.clone(),
            }],
        });
        i += 1;
    }
    segments
}

fn output_format(
    segments: Vec<OriginSegment>,
    format: OutputFormat,
    separator: &str,
) -> SegmentOutput {
    match format {
        OutputFormat::AllSegment => SegmentOutput::AllSegment(
            segments
                .into_iter()
                .map(|s| SegmentPair {
                    origin: s
                        .segment
                        .iter()
                        .map(|p| p.origin.clone())
                        .collect::<Vec<_>>()
                        .join(""),
                    result: s
                        .segment
                        .iter()
                        .map(|p| p.result.clone())
                        .collect::<Vec<_>>()
                        .join(""),
                })
                .collect(),
        ),
        OutputFormat::AllArray => {
            SegmentOutput::AllArray(segments.into_iter().map(|s| s.segment).collect())
        }
        OutputFormat::AllString => {
            let pairs: Vec<SegmentPair> = segments
                .into_iter()
                .map(|s| SegmentPair {
                    origin: s
                        .segment
                        .iter()
                        .map(|p| p.origin.clone())
                        .collect::<Vec<_>>()
                        .join(""),
                    result: s
                        .segment
                        .iter()
                        .map(|p| p.result.clone())
                        .collect::<Vec<_>>()
                        .join(""),
                })
                .collect();
            SegmentOutput::AllString(SegmentPair {
                origin: pairs
                    .iter()
                    .map(|p| p.origin.clone())
                    .collect::<Vec<_>>()
                    .join(separator),
                result: pairs
                    .iter()
                    .map(|p| p.result.clone())
                    .collect::<Vec<_>>()
                    .join(separator),
            })
        }
        OutputFormat::PinyinSegment => SegmentOutput::PinyinSegment(
            segments
                .into_iter()
                .map(|s| {
                    s.segment
                        .iter()
                        .map(|p| p.result.clone())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect(),
        ),
        OutputFormat::PinyinArray => SegmentOutput::PinyinArray(
            segments
                .into_iter()
                .map(|s| s.segment.iter().map(|p| p.result.clone()).collect())
                .collect(),
        ),
        OutputFormat::PinyinString => SegmentOutput::PinyinString(
            segments
                .into_iter()
                .map(|s| {
                    s.segment
                        .iter()
                        .map(|p| p.result.clone())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect::<Vec<_>>()
                .join(separator),
        ),
        OutputFormat::ZhSegment => SegmentOutput::ZhSegment(
            segments
                .into_iter()
                .map(|s| {
                    s.segment
                        .iter()
                        .map(|p| p.origin.clone())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect(),
        ),
        OutputFormat::ZhArray => SegmentOutput::ZhArray(
            segments
                .into_iter()
                .map(|s| s.segment.iter().map(|p| p.origin.clone()).collect())
                .collect(),
        ),
        OutputFormat::ZhString => SegmentOutput::ZhString(
            segments
                .into_iter()
                .map(|s| {
                    s.segment
                        .iter()
                        .map(|p| p.origin.clone())
                        .collect::<Vec<_>>()
                        .join("")
                })
                .collect::<Vec<_>>()
                .join(separator),
        ),
    }
}

/// Mirrors `segment(word, options)`.
pub fn segment(text: &str, options: SegmentOptions) -> SegmentOutput {
    if text.is_empty() {
        // JS: validateType passes for "", then getPinyin on empty, middlewares
        // on empty lists, and formatting of empty segments.
        return output_format(Vec::new(), options.format, &options.separator);
    }
    let surname = options.surname.or_else(|| {
        Some(if options.mode == PinyinMode::Surname {
            SurnameMode::All
        } else {
            SurnameMode::Off
        })
    });
    let word_chars: Vec<char> = text.chars().collect();
    let (list0, matches) = get_pinyin_with_algo(
        &word_chars,
        surname.unwrap_or(SurnameMode::Off),
        options.segmentit,
        options.traditional,
    );
    let mut list = list0;
    middleware_tone_sandhi(&mut list, options.tone_sandhi);
    let scope = compile_scope(&options.non_zh_scope);
    list = middleware_non_zh(list, options.non_zh, &scope);
    // toneType / v via a PinyinOptions view
    let view = PinyinOptions {
        tone_type: options.tone_type,
        v: options.v.clone(),
        ..Default::default()
    };
    middleware_tone_type(&mut list, &view);
    middleware_v(&mut list, &options.v);
    let segments = middleware_segment(&list, &matches);
    output_format(segments, options.format, &options.separator)
}
