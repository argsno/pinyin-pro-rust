mod helpers;

use helpers::lock;
use pinyin_pro::custom::{clear_custom_dict, custom_pinyin, CustomDictKind};
use pinyin_pro::dict_api::{add_dict, remove_dict, AddDictOptions, DictValue};
use pinyin_pro::options::*;
use pinyin_pro::pinyin::pinyin;
use pinyin_pro::traditional::add_traditional_dict;

fn s(text: &str, o: PinyinOptions) -> String {
    match pinyin(text, o) {
        PinyinOutput::Str(x) => x,
        _ => panic!("expected string"),
    }
}

#[test]
fn custom_pinyin_words() {
    let _g = lock();
    custom_pinyin(
        &[("哈什玛".to_string(), "hà shén mǎ".to_string())],
        Default::default(),
    );
    assert_eq!(s("哈什玛", PinyinOptions::default()), "hà shén mǎ");
    clear_custom_dict(&[CustomDictKind::Pinyin]);
    // back to single-char readings after clearing
    assert_eq!(s("哈什玛", PinyinOptions::default()), "hā shén mǎ");
}

#[test]
fn custom_single_char_override() {
    let _g = lock();
    custom_pinyin(&[("能".to_string(), "nài".to_string())], Default::default());
    assert_eq!(s("我姓能", PinyinOptions::default()), "wǒ xìng nài");
    clear_custom_dict(&[CustomDictKind::Pinyin]);
    assert_eq!(s("我姓能", PinyinOptions::default()), "wǒ xìng néng");
}

#[test]
fn custom_multiple_and_polyphonic() {
    let _g = lock();
    use pinyin_pro::custom::{CustomHandle, CustomPinyinOptions};
    // per-char supplement: 你 += mi, 好 += kao
    custom_pinyin(
        &[("你好".to_string(), "mi kao".to_string())],
        CustomPinyinOptions {
            multiple: Some(CustomHandle::Add),
            polyphonic: None,
        },
    );
    assert_eq!(
        s(
            "你",
            PinyinOptions {
                multiple: true,
                ..Default::default()
            }
        ),
        "nǐ mi"
    );
    assert_eq!(
        s(
            "好",
            PinyinOptions {
                multiple: true,
                ..Default::default()
            }
        ),
        "hǎo hào kao"
    );
    // replace overwrites
    custom_pinyin(
        &[("你好".to_string(), "mi kao".to_string())],
        CustomPinyinOptions {
            multiple: Some(CustomHandle::Replace),
            polyphonic: None,
        },
    );
    assert_eq!(
        s(
            "好",
            PinyinOptions {
                multiple: true,
                ..Default::default()
            }
        ),
        "kao"
    );
    clear_custom_dict(&[CustomDictKind::Multiple, CustomDictKind::Polyphonic]);
    assert_eq!(
        s(
            "好",
            PinyinOptions {
                multiple: true,
                ..Default::default()
            }
        ),
        "hǎo hào"
    );
    // polyphonic add supplements every char's reading list
    custom_pinyin(
        &[("你好".to_string(), "mi kao".to_string())],
        CustomPinyinOptions {
            multiple: None,
            polyphonic: Some(CustomHandle::Add),
        },
    );
    match pinyin_pro::polyphonic("好好学习", PolyphonicOptions::default()) {
        PolyphonicOutput::Str(v) => {
            assert_eq!(v, ["hǎo hào kao", "hǎo hào kao", "xué", "xí"])
        }
        _ => panic!("expected str"),
    }
    clear_custom_dict(&[
        CustomDictKind::Pinyin,
        CustomDictKind::Multiple,
        CustomDictKind::Polyphonic,
    ]);
}

#[test]
fn add_and_remove_dict() {
    let _g = lock();
    add_dict(
        &[
            ("新词X".to_string(), DictValue::from("xīn cí")),
            (
                "测试词".to_string(),
                DictValue {
                    pinyin: "cè shì cí".to_string(),
                    probability: Some(0.5),
                    pos: Some("n".to_string()),
                    is_array: true,
                },
            ),
        ],
        AddDictOptions::default(),
    );
    assert_eq!(
        s("这是新词X和测试词", PinyinOptions::default()),
        "zhè shì xīn cí  hé cè shì cí"
    );
    remove_dict(None);
    // X falls back to a passthrough token after removal
    assert_eq!(
        s("这是新词X和测试词", PinyinOptions::default()),
        "zhè shì xīn cí X hé cè shì cí"
    );
}

#[test]
fn add_dict_single_char_replace() {
    let _g = lock();
    add_dict(
        &[("好".to_string(), DictValue::from("hǎo3"))],
        AddDictOptions {
            dict1: pinyin_pro::dict_api::Dict1Handle::Replace,
            ..Default::default()
        },
    );
    assert_eq!(s("好人", PinyinOptions::default()), "hǎo3 rén");
    remove_dict(None);
    assert_eq!(s("好人", PinyinOptions::default()), "hǎo rén");
}

#[test]
fn traditional_mode() {
    let _g = lock();
    add_traditional_dict(&[
        ("國".to_string(), "国".to_string()),
        ("人".to_string(), "人".to_string()),
    ]);
    assert_eq!(
        s(
            "中國人",
            PinyinOptions {
                traditional: true,
                ..Default::default()
            }
        ),
        "zhōng guó rén"
    );
}
