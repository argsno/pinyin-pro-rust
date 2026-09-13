mod helpers;

use helpers::lock;
use pinyin_pro::convert::{convert, convert_array};
use pinyin_pro::options::*;
use pinyin_pro::polyphonic::polyphonic;

#[test]
fn polyphonic_str_and_array() {
    let _g = lock();
    match polyphonic("好好学习", PolyphonicOptions::default()) {
        PolyphonicOutput::Str(v) => {
            assert_eq!(v, ["hǎo hào", "hǎo hào", "xué", "xí"])
        }
        _ => panic!("expected str"),
    }
    match polyphonic(
        "好好学习",
        PolyphonicOptions {
            type_mode: TypeMode::Array,
            ..Default::default()
        },
    ) {
        PolyphonicOutput::Arr(v) => assert_eq!(
            v,
            vec![
                vec!["hǎo".to_string(), "hào".to_string()],
                vec!["hǎo".to_string(), "hào".to_string()],
                vec!["xué".to_string()],
                vec!["xí".to_string()],
            ]
        ),
        _ => panic!("expected array"),
    }
    // empty + non-string-safe inputs
    match polyphonic("", PolyphonicOptions::default()) {
        PolyphonicOutput::Arr(v) => assert!(v.is_empty()),
        _ => panic!("expected empty array"),
    }
    // non-zh passthrough + removal
    match polyphonic("好好学习s", PolyphonicOptions::default()) {
        PolyphonicOutput::Str(v) => {
            assert_eq!(v, ["hǎo hào", "hǎo hào", "xué", "xí", "s"])
        }
        _ => panic!("expected str"),
    }
    match polyphonic(
        "好好学习s",
        PolyphonicOptions {
            remove_non_zh: true,
            ..Default::default()
        },
    ) {
        PolyphonicOutput::Str(v) => {
            assert_eq!(v, ["hǎo hào", "hǎo hào", "xué", "xí"])
        }
        _ => panic!("expected str"),
    }
}

#[test]
fn polyphonic_all() {
    let _g = lock();
    match polyphonic(
        "学",
        PolyphonicOptions {
            type_mode: TypeMode::All,
            ..Default::default()
        },
    ) {
        PolyphonicOutput::All(v) => {
            assert_eq!(v.len(), 1);
            assert_eq!(v[0].len(), 1);
            let i = &v[0][0];
            assert_eq!(i.origin, "学");
            assert_eq!(i.pinyin, "xué");
            assert_eq!(i.initial, "x");
            assert_eq!(i.final_, "üé");
            assert_eq!(i.num, 2);
            assert!(i.is_zh && i.in_zh_range);
        }
        _ => panic!("expected all"),
    }
}

#[test]
fn convert_formats() {
    let _g = lock();
    let d = ConvertOptions::default();
    assert_eq!(convert("pin1 yin1", d.clone()), "pīn yīn");
    assert_eq!(
        convert(
            "pīn yīn",
            ConvertOptions {
                format: ConvertFormat::SymbolToNum,
                ..d.clone()
            }
        ),
        "pin1 yin1"
    );
    assert_eq!(
        convert(
            "pīn yīn",
            ConvertOptions {
                format: ConvertFormat::ToneNone,
                ..d.clone()
            }
        ),
        "pin yin"
    );
    assert_eq!(convert("lv4", d.clone()), "lǜ");
    assert_eq!(
        convert(
            "lǜ",
            ConvertOptions {
                format: ConvertFormat::SymbolToNum,
                ..d.clone()
            }
        ),
        "lü4"
    );
    // erhua
    assert_eq!(convert("dian3r", d.clone()), "diǎnr");
    assert_eq!(
        convert(
            "diǎnr",
            ConvertOptions {
                format: ConvertFormat::SymbolToNum,
                ..d.clone()
            }
        ),
        "dian3r"
    );
    // custom separator + array form
    assert_eq!(
        convert(
            "zhong1-guo2",
            ConvertOptions {
                separator: "-".to_string(),
                ..d.clone()
            }
        ),
        "zhōng-guó"
    );
    assert_eq!(
        convert_array(
            vec!["pin1".to_string(), "yin1".to_string()],
            ConvertOptions::default()
        ),
        ["pīn".to_string(), "yīn".to_string()]
    );
}
