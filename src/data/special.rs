//! Tables from `lib/data/special.ts`.

/// Sound initials, longest first. Note `""` matches everything.
pub static INITIAL_LIST: &[&str] = &[
    "zh", "ch", "sh", "z", "c", "s", "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h", "j",
    "q", "x", "r", "y", "w", "",
];

pub static SPECIAL_INITIAL_LIST: &[&str] = &["j", "q", "x"];

pub static SPECIAL_FINAL_LIST: &[&str] = &[
    "uān", "uán", "uǎn", "uàn", "uan", "uē", "ué", "uě", "uè", "ue", "ūn", "ún", "ǔn", "ùn", "un",
    "ū", "ú", "ǔ", "ù", "u",
];

/// (plain, ü-form) pairs for the j/q/x special-case.
pub static SPECIAL_FINAL_MAP: &[(&str, &str)] = &[
    ("uān", "üān"),
    ("uán", "üán"),
    ("uǎn", "üǎn"),
    ("uàn", "üàn"),
    ("uan", "üan"),
    ("uē", "üē"),
    ("ué", "üé"),
    ("uě", "üě"),
    ("uè", "üè"),
    ("ue", "üe"),
    ("ūn", "ǖn"),
    ("ún", "ǘn"),
    ("ǔn", "ǚn"),
    ("ùn", "ǜn"),
    ("un", "ün"),
    ("ū", "ǖ"),
    ("ú", "ǘ"),
    ("ǔ", "ǚ"),
    ("ù", "ǜ"),
    ("u", "ü"),
];

/// Finals (toneless) that split into head + body + tail.
pub static DOUBLE_FINAL_LIST: &[&str] = &[
    "ia", "ian", "iang", "iao", "ie", "iu", "iong", "ua", "uai", "uan", "uang", "ue", "ui", "uo",
    "üan", "üe", "van", "ve",
];

/// Number readings used by `genNumberDict()`, in JS object order.
pub static NUMBERS: &[(&str, &str)] = &[
    ("一", "yì"),
    ("二", "èr"),
    ("三", "sān"),
    ("四", "sì"),
    ("五", "wǔ"),
    ("六", "liù"),
    ("七", "qī"),
    ("八", "bā"),
    ("九", "jiǔ"),
    ("十", "shí"),
    ("百", "bǎi"),
    ("千", "qiān"),
    ("万", "wàn"),
    ("亿", "yì"),
    ("单", "dān"),
    ("两", "liǎng"),
    ("双", "shuāng"),
    ("多", "duō"),
    ("几", "jǐ"),
    ("十一", "shí yī"),
    ("零一", "líng yī"),
    ("第一", "dì yī"),
    ("一十", "yī shí"),
    ("一十一", "yī shí yī"),
];

pub static NUMBER_WORD_MAP: &[(&str, &str)] = &[
    ("重", "chóng"),
    ("行", "háng"),
    ("斗", "dǒu"),
    ("更", "gēng"),
];

/// Tone-sandhi map for 一/不: reading -> following tones triggering it.
pub static TONE_SANDHI_BU: &[(&str, &[i64])] = &[("bú", &[4])];
pub static TONE_SANDHI_YI: &[(&str, &[i64])] = &[("yí", &[4]), ("yì", &[1, 2, 3])];

pub static TONE_SANDHI_IGNORE_BU: &[&str] = &["的", "而", "之", "后", "也", "还", "地"];
pub static TONE_SANDHI_IGNORE_YI: &[&str] = &["的", "而", "之", "后", "也", "还", "是"];

pub fn is_tone_sandhi_char(c: char) -> bool {
    c == '一' || c == '不'
}
