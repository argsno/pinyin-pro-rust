mod helpers;

use helpers::lock;
use pinyin_pro::html::html;
use pinyin_pro::match_::match_text;
use pinyin_pro::options::*;
use pinyin_pro::segment::segment;

fn m(text: &str, query: &str) -> Option<Vec<usize>> {
    match_text(text, query, MatchOptions::default())
}

#[test]
fn match_basics() {
    let _g = lock();
    assert_eq!(m("欢迎使用汉语拼音", "hy"), Some(vec![0, 1]));
    assert_eq!(m("汉语拼音", "hanpin"), Some(vec![0, 2]));
    assert_eq!(m("汉语拼音", "hyupy"), Some(vec![0, 1, 2, 3]));
    assert_eq!(m("开会", "kaig"), None);
    assert_eq!(m("开会", "l"), None);
    assert_eq!(m("会计", "kj"), Some(vec![0, 1]));
    assert_eq!(m("会计", "huij"), Some(vec![0, 1]));
}

#[test]
fn match_options() {
    let _g = lock();
    // start + continuous
    assert_eq!(
        match_text(
            "欢迎使用汉语拼音",
            "yingshyon",
            MatchOptions {
                precision: MatchPrecision::Start,
                continuous: true,
                ..Default::default()
            }
        ),
        Some(vec![1, 2, 3])
    );
    // any
    assert_eq!(
        match_text(
            "开会",
            "kaiui",
            MatchOptions {
                precision: MatchPrecision::Any,
                ..Default::default()
            }
        ),
        Some(vec![0, 1])
    );
    assert_eq!(
        match_text(
            "开会",
            "",
            MatchOptions {
                precision: MatchPrecision::Any,
                ..Default::default()
            }
        ),
        None
    );
    // lastPrecision every
    assert_eq!(
        match_text(
            "汉语拼音",
            "hanyupinyin",
            MatchOptions {
                last_precision: MatchPrecision::Every,
                ..Default::default()
            }
        ),
        Some(vec![0, 1, 2, 3])
    );
    assert_eq!(
        match_text(
            "汉语拼音",
            "hanyupinyi",
            MatchOptions {
                last_precision: MatchPrecision::Every,
                ..Default::default()
            }
        ),
        None
    );
    // insensitive
    assert_eq!(
        m("汉语KK拼音", "hanyukkpinyin"),
        Some(vec![0, 1, 2, 3, 4, 5])
    );
    assert_eq!(
        match_text(
            "汉语KK拼音",
            "hanyukkpinyin",
            MatchOptions {
                insensitive: false,
                ..Default::default()
            }
        ),
        None
    );
    // v
    assert_eq!(
        match_text(
            "我是吕布",
            "woshilvbu",
            MatchOptions {
                v: VMode::V,
                ..Default::default()
            }
        ),
        Some(vec![0, 1, 2, 3])
    );
    assert_eq!(
        match_text(
            "吕",
            "lü",
            MatchOptions {
                v: VMode::Custom("u".to_string()),
                ..Default::default()
            }
        ),
        Some(vec![0])
    );
    // tone marks in query never match (only dict side is stripped)
    assert_eq!(m("女", "nv"), None);
    assert_eq!(m("女", "nü"), Some(vec![0]));
    assert_eq!(m("嗯", "n"), Some(vec![0]));
    assert_eq!(m("嗯", "ń"), None);
    // invalid precision strings match nothing extra
    assert_eq!(
        match_text(
            "汉语拼音",
            "hanyupini",
            MatchOptions {
                last_precision: MatchPrecision::Invalid,
                ..Default::default()
            }
        ),
        None
    );
}

#[test]
fn html_output() {
    let _g = lock();
    let out = html("汉", HtmlOptions::default());
    assert_eq!(
    out,
    "<span class=\"py-result-item\"><ruby><span class=\"py-chinese-item\">汉</span><rp>(</rp><rt class=\"py-pinyin-item\">hàn</rt><rp>)</rp></ruby></span>"
  );
    // non-zh passthrough
    assert_eq!(html("a", HtmlOptions::default()), "a");
    // wrapped non-zh, no rp
    assert_eq!(
        html(
            "a",
            HtmlOptions {
                wrap_non_chinese: true,
                rp: false,
                ..Default::default()
            }
        ),
        "<span class=\"py-non-chinese-item\">a</span>"
    );
    // custom classes incl. non-zh target
    let out = html(
        "汉语，拼音",
        HtmlOptions {
            custom_class_map: vec![("hl".to_string(), vec!["汉".to_string()])],
            ..Default::default()
        },
    );
    assert!(out.contains("<span class=\"py-result-item hl\">"));
}

#[test]
fn segment_output() {
    let _g = lock();
    // AllSegment default: single chars stay split without a word match
    // covering them from the current position (mirrors JS exactly).
    match segment("汉语拼音", SegmentOptions::default()) {
        SegmentOutput::AllSegment(v) => {
            assert_eq!(v.len(), 4);
            assert_eq!(v[0].origin, "汉");
            assert_eq!(v[0].result, "hàn");
        }
        _ => panic!("expected AllSegment"),
    }
    // surname words group: 令狐 -> línghú
    match segment("我叫令狐冲", SegmentOptions::default()) {
        SegmentOutput::AllSegment(v) => {
            assert_eq!(v.len(), 4);
            assert_eq!(v[2].origin, "令狐");
            assert_eq!(v[2].result, "línghú");
        }
        _ => panic!("expected AllSegment"),
    }
    match segment(
        "汉语拼音",
        SegmentOptions {
            format: OutputFormat::PinyinString,
            ..Default::default()
        },
    ) {
        SegmentOutput::PinyinString(x) => assert_eq!(x, "hàn yǔ pīn yīn"),
        _ => panic!("expected PinyinString"),
    }
    match segment(
        "汉语拼音",
        SegmentOptions {
            format: OutputFormat::ZhString,
            separator: "-".to_string(),
            ..Default::default()
        },
    ) {
        SegmentOutput::ZhString(x) => assert_eq!(x, "汉-语-拼-音"),
        _ => panic!("expected ZhString"),
    }
    match segment(
        "我叫令狐冲",
        SegmentOptions {
            mode: PinyinMode::Surname,
            format: OutputFormat::PinyinSegment,
            ..Default::default()
        },
    ) {
        SegmentOutput::PinyinSegment(v) => {
            assert!(v.contains(&"línghú".to_string()), "got {v:?}")
        }
        _ => panic!("expected PinyinSegment"),
    }
}
