mod helpers;

use helpers::lock;
use pinyin_pro::options::*;
use pinyin_pro::pinyin::pinyin;
use pinyin_pro::segmentit::TokenizationAlgorithm;

fn s(text: &str) -> String {
    match pinyin(text, PinyinOptions::default()) {
        PinyinOutput::Str(x) => x,
        _ => panic!("expected string"),
    }
}

fn arr(text: &str, o: PinyinOptions) -> Vec<String> {
    match pinyin(text, o) {
        PinyinOutput::Arr(x) => x,
        _ => panic!("expected array"),
    }
}

fn opt() -> PinyinOptions {
    PinyinOptions::default()
}

#[test]
fn basic_string() {
    let _g = lock();
    assert_eq!(s("汉语拼音"), "hàn yǔ pīn yīn");
    assert_eq!(s("汉语拼音xxx.,"), "hàn yǔ pīn yīn x x x . ,");
    // without the extended complete dict, 好学 wins over 好好 (same as JS)
    assert_eq!(s("好好学习"), "hǎo hào xué xí");
    assert_eq!(s(""), "");
    assert_eq!(s("哈发生你看三零四"), "hā fā shēng nǐ kàn sān líng sì");
}

#[test]
fn basic_array() {
    let _g = lock();
    assert_eq!(
        arr(
            "汉语拼音",
            PinyinOptions {
                type_mode: TypeMode::Array,
                ..opt()
            }
        ),
        ["hàn", "yǔ", "pīn", "yīn"]
    );
    assert_eq!(
        arr(
            "",
            PinyinOptions {
                type_mode: TypeMode::Array,
                ..opt()
            }
        ),
        Vec::<String>::new()
    );
}

#[test]
fn tone_types() {
    let _g = lock();
    let o = |t: ToneType| PinyinOptions {
        tone_type: t,
        ..opt()
    };
    assert_eq!(s("汉语拼音"), "hàn yǔ pīn yīn");
    assert_eq!(
        match pinyin("汉语拼音", o(ToneType::None)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "han yu pin yin"
    );
    assert_eq!(
        match pinyin("汉语拼音", o(ToneType::Num)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "han4 yu3 pin1 yin1"
    );
    // pattern num implies toneType none
    assert_eq!(
        match pinyin(
            "汉语拼音",
            PinyinOptions {
                pattern: PatternKind::Num,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "4 3 1 1"
    );
}

#[test]
fn patterns() {
    let _g = lock();
    let o = |p: PatternKind| PinyinOptions {
        pattern: p,
        ..opt()
    };
    assert_eq!(
        match pinyin("汉语拼音", o(PatternKind::Initial)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "h y p y"
    );
    assert_eq!(
        match pinyin("汉语拼音", o(PatternKind::Final)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "àn ǔ īn īn"
    );
    assert_eq!(
        match pinyin("汉语拼音", o(PatternKind::First)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "h y p y"
    );
    assert_eq!(
        match pinyin("庄", o(PatternKind::FinalHead)) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "u"
    );
    assert_eq!(
        match pinyin(
            "汉语拼音",
            PinyinOptions {
                pattern: PatternKind::Initial,
                tone_type: ToneType::Num,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "h4 y3 p1 y1"
    );
}

#[test]
fn multiple_mode() {
    let _g = lock();
    assert_eq!(s("好"), "hǎo");
    assert_eq!(
        match pinyin(
            "好",
            PinyinOptions {
                multiple: true,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hǎo hào"
    );
    // multiple only applies to single chars
    assert_eq!(
        match pinyin(
            "汉语拼音",
            PinyinOptions {
                multiple: true,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hàn yǔ pīn yīn"
    );
    // dedup after tone removal
    assert_eq!(
        match pinyin(
            "好",
            PinyinOptions {
                multiple: true,
                tone_type: ToneType::None,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hao"
    );
}

#[test]
fn surname_mode() {
    let _g = lock();
    let o = PinyinOptions {
        mode: PinyinMode::Surname,
        ..opt()
    };
    assert_eq!(
        match pinyin("万俟", o.clone()) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "mò qí"
    );
    assert_eq!(
        match pinyin("我叫令狐冲", o.clone()) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "wǒ jiào líng hú chōng"
    );
    assert_eq!(
        match pinyin(
            "曾乐乐",
            PinyinOptions {
                mode: PinyinMode::Surname,
                surname: Some(SurnameMode::Head),
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "zēng lè lè"
    );
    // multiple + surname: surname reading first
    assert_eq!(
        match pinyin(
            "能",
            PinyinOptions {
                mode: PinyinMode::Surname,
                multiple: true,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "nài néng"
    );
}

#[test]
fn tone_sandhi() {
    let _g = lock();
    // 一 before 4th tone -> 2nd tone; 不 before 4th tone -> 2nd tone
    assert_eq!(s("一个"), "yí gè");
    assert_eq!(s("不要"), "bú yào");
    assert_eq!(s("一本书"), "yì běn shū");
    assert_eq!(
        match pinyin(
            "不是",
            PinyinOptions {
                tone_sandhi: false,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "bù shì"
    );
    assert_eq!(s("说一说"), "shuō yi shuō");
    assert_eq!(s("展々"), "zhǎn zhǎn");
}

#[test]
fn non_zh_handling() {
    let _g = lock();
    assert_eq!(s("汉语，拼音"), "hàn yǔ ， pīn yīn");
    assert_eq!(
        match pinyin(
            "汉语，拼音",
            PinyinOptions {
                non_zh: NonZh::Consecutive,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hàn yǔ ， pīn yīn"
    );
    assert_eq!(
        match pinyin(
            "ab汉语",
            PinyinOptions {
                non_zh: NonZh::Consecutive,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "ab hàn yǔ"
    );
    assert_eq!(
        match pinyin(
            "ab汉语",
            PinyinOptions {
                remove_non_zh: true,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hàn yǔ"
    );
    // scoped removal keeps unmatched non-zh
    assert_eq!(
        match pinyin(
            "汉语abc拼音，。",
            PinyinOptions {
                non_zh: NonZh::Removed,
                non_zh_scope: Some("[a-z]".to_string()),
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hàn yǔ pīn yīn ， 。"
    );
}

#[test]
fn separator_and_v() {
    let _g = lock();
    assert_eq!(
        match pinyin(
            "汉语拼音",
            PinyinOptions {
                separator: "-".to_string(),
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "hàn-yǔ-pīn-yīn"
    );
    assert_eq!(
        match pinyin(
            "吕",
            PinyinOptions {
                tone_type: ToneType::None,
                v: VMode::V,
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "lv"
    );
    assert_eq!(
        match pinyin(
            "吕",
            PinyinOptions {
                tone_type: ToneType::None,
                v: VMode::Custom("u:".to_string()),
                ..opt()
            }
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        },
        "lu:"
    );
}

#[test]
fn type_all() {
    let _g = lock();
    match pinyin(
        "汉",
        PinyinOptions {
            type_mode: TypeMode::All,
            ..opt()
        },
    ) {
        PinyinOutput::All(all) => {
            assert_eq!(all.len(), 1);
            let a = &all[0];
            assert_eq!(a.origin, "汉");
            assert_eq!(a.pinyin, "hàn");
            assert_eq!(a.initial, "h");
            assert_eq!(a.final_, "àn");
            assert_eq!(a.num, 4);
            assert_eq!(a.first, "h");
            assert!(a.is_zh && a.in_zh_range);
            assert_eq!(a.result, "hàn");
        }
        _ => panic!("expected all"),
    }
}

#[test]
fn segmentit_algorithms_agree_on_common() {
    let _g = lock();
    for algo in [
        TokenizationAlgorithm::ReverseMaxMatch,
        TokenizationAlgorithm::MaxProbability,
        TokenizationAlgorithm::MinTokenization,
    ] {
        let out = match pinyin(
            "汉语拼音",
            PinyinOptions {
                segmentit: algo,
                ..opt()
            },
        ) {
            PinyinOutput::Str(x) => x,
            _ => unreachable!(),
        };
        assert_eq!(out, "hàn yǔ pīn yīn");
    }
}

#[test]
fn pure_fns() {
    let _g = lock();
    use pinyin_pro::pinyin_utils::*;
    assert_eq!(num_of_tone("hàn yǔ"), "4 3");
    assert_eq!(strip_tone("hàn yǔ"), "han yu");
    assert_eq!(strip_tone("ǚ"), "ü");
    let (i, f) = initial_and_final("xué", InitialPattern::Yw);
    assert_eq!((i.as_str(), f.as_str()), ("x", "üé"));
    let (i, _) = initial_and_final("zhuāng", InitialPattern::Standard);
    assert_eq!(i, "zh");
    let (h, b, t) = final_parts("zhuāng");
    assert_eq!((h.as_str(), b.as_str(), t.as_str()), ("u", "ā", "ng"));
}
