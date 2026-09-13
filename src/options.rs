//! Public option and output types mirroring `lib/common/type.ts` and the
//! per-API option interfaces.

use crate::segmentit::TokenizationAlgorithm;

// ---------------------------------------------------------------------------
// shared enums
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ToneType {
    #[default]
    Symbol,
    Num,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PatternKind {
    #[default]
    Pinyin,
    Initial,
    Final,
    Num,
    First,
    FinalHead,
    FinalBody,
    FinalTail,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum NonZh {
    #[default]
    Spaced,
    Consecutive,
    Removed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SurnameMode {
    All,
    Head,
    #[default]
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PinyinMode {
    #[default]
    Normal,
    Surname,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum InitialPattern {
    #[default]
    Yw,
    Standard,
}

/// `v` option: how to render `ü` (only applies when tone is removed).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum VMode {
    #[default]
    Off,
    V,
    Custom(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TypeMode {
    #[default]
    Str,
    Array,
    All,
}

// ---------------------------------------------------------------------------
// pinyin()
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct PinyinOptions {
    pub pattern: PatternKind,
    pub tone_type: ToneType,
    pub type_mode: TypeMode,
    pub multiple: bool,
    pub mode: PinyinMode,
    pub surname: Option<SurnameMode>,
    pub tone_sandhi: bool,
    pub segmentit: TokenizationAlgorithm,
    pub non_zh: NonZh,
    pub non_zh_scope: Option<String>,
    pub remove_non_zh: bool,
    pub v: VMode,
    pub separator: String,
    pub initial_pattern: InitialPattern,
    pub traditional: bool,
}

impl Default for PinyinOptions {
    fn default() -> Self {
        PinyinOptions {
            pattern: PatternKind::Pinyin,
            tone_type: ToneType::Symbol,
            type_mode: TypeMode::Str,
            multiple: false,
            mode: PinyinMode::Normal,
            surname: None,
            tone_sandhi: true,
            segmentit: TokenizationAlgorithm::MaxProbability,
            non_zh: NonZh::Spaced,
            non_zh_scope: None,
            remove_non_zh: false,
            v: VMode::Off,
            separator: " ".to_string(),
            initial_pattern: InitialPattern::Yw,
            traditional: false,
        }
    }
}

impl PinyinOptions {
    /// Apply the same normalizations as `pinyin()` in `core/pinyin/index.ts`.
    pub fn normalized(mut self) -> Self {
        if self.surname.is_none() {
            self.surname = Some(if self.mode == PinyinMode::Surname {
                SurnameMode::All
            } else {
                SurnameMode::Off
            });
        }
        if self.type_mode == TypeMode::All {
            self.pattern = PatternKind::Pinyin;
        }
        if self.pattern == PatternKind::Num {
            self.tone_type = ToneType::None;
        }
        if self.remove_non_zh {
            self.non_zh = NonZh::Removed;
        }
        self
    }

    pub fn surname(&self) -> SurnameMode {
        self.surname.unwrap_or(SurnameMode::Off)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllInfo {
    pub origin: String,
    pub pinyin: String,
    pub initial: String,
    pub final_: String,
    pub num: i64,
    pub first: String,
    pub final_head: String,
    pub final_body: String,
    pub final_tail: String,
    pub is_zh: bool,
    pub polyphonic: Vec<String>,
    pub in_zh_range: bool,
    pub result: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PinyinOutput {
    Str(String),
    Arr(Vec<String>),
    All(Vec<AllInfo>),
}

// ---------------------------------------------------------------------------
// polyphonic()
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct PolyphonicOptions {
    pub pattern: PatternKind,
    pub tone_type: ToneType,
    pub type_mode: TypeMode,
    pub non_zh: NonZh,
    pub non_zh_scope: Option<String>,
    pub remove_non_zh: bool,
    pub v: VMode,
    pub initial_pattern: InitialPattern,
}

impl PolyphonicOptions {
    pub fn normalized(mut self) -> Self {
        if self.type_mode == TypeMode::All {
            self.pattern = PatternKind::Pinyin;
        }
        if self.pattern == PatternKind::Num {
            self.tone_type = ToneType::None;
        }
        if self.remove_non_zh {
            self.non_zh = NonZh::Removed;
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolyInfo {
    pub origin: String,
    pub pinyin: String,
    pub initial: String,
    pub final_: String,
    pub num: i64,
    pub first: String,
    pub final_head: String,
    pub final_body: String,
    pub final_tail: String,
    pub is_zh: bool,
    pub in_zh_range: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolyphonicOutput {
    /// type string: one joined string per char
    Str(Vec<String>),
    /// type array: readings per char
    Arr(Vec<Vec<String>>),
    All(Vec<Vec<PolyInfo>>),
}

// ---------------------------------------------------------------------------
// match()
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MatchPrecision {
    #[default]
    First,
    Start,
    Every,
    Any,
    /// Unknown precision string: only complete-pinyin matching applies
    /// (for `lastPrecision` it never matches).
    Invalid,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MatchSpace {
    #[default]
    Ignore,
    Preserve,
}

#[derive(Clone, Debug)]
pub struct MatchOptions {
    pub precision: MatchPrecision,
    pub continuous: bool,
    pub space: MatchSpace,
    pub last_precision: MatchPrecision,
    pub insensitive: bool,
    pub v: VMode,
}

impl Default for MatchOptions {
    fn default() -> Self {
        MatchOptions {
            precision: MatchPrecision::First,
            continuous: false,
            space: MatchSpace::Ignore,
            last_precision: MatchPrecision::Start,
            insensitive: true,
            v: VMode::Off,
        }
    }
}

// ---------------------------------------------------------------------------
// convert()
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ConvertFormat {
    #[default]
    NumToSymbol,
    SymbolToNum,
    ToneNone,
}

#[derive(Clone, Debug)]
pub struct ConvertOptions {
    pub separator: String,
    pub format: ConvertFormat,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        ConvertOptions {
            separator: " ".to_string(),
            format: ConvertFormat::NumToSymbol,
        }
    }
}

// ---------------------------------------------------------------------------
// html()
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct HtmlOptions {
    pub tone_type: ToneType,
    pub v: VMode,
    pub tone_sandhi: bool,
    pub segmentit: TokenizationAlgorithm,
    pub traditional: bool,
    pub surname: Option<SurnameMode>,
    pub mode: PinyinMode,
    pub result_class: String,
    pub pinyin_class: String,
    pub chinese_class: String,
    pub wrap_non_chinese: bool,
    pub non_chinese_class: String,
    pub custom_class_map: Vec<(String, Vec<String>)>,
    pub rp: bool,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        HtmlOptions {
            tone_type: ToneType::Symbol,
            v: VMode::Off,
            tone_sandhi: true,
            segmentit: TokenizationAlgorithm::MaxProbability,
            traditional: false,
            surname: None,
            mode: PinyinMode::Normal,
            result_class: "py-result-item".to_string(),
            pinyin_class: "py-pinyin-item".to_string(),
            chinese_class: "py-chinese-item".to_string(),
            wrap_non_chinese: false,
            non_chinese_class: "py-non-chinese-item".to_string(),
            custom_class_map: Vec::new(),
            rp: true,
        }
    }
}

// ---------------------------------------------------------------------------
// segment()
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum OutputFormat {
    #[default]
    AllSegment = 1,
    AllArray = 2,
    AllString = 3,
    PinyinSegment = 4,
    PinyinArray = 5,
    PinyinString = 6,
    ZhSegment = 7,
    ZhArray = 8,
    ZhString = 9,
}

#[derive(Clone, Debug)]
pub struct SegmentOptions {
    pub tone_type: ToneType,
    pub mode: PinyinMode,
    pub surname: Option<SurnameMode>,
    pub non_zh: NonZh,
    pub non_zh_scope: Option<String>,
    pub v: VMode,
    pub tone_sandhi: bool,
    pub segmentit: TokenizationAlgorithm,
    pub traditional: bool,
    pub format: OutputFormat,
    pub separator: String,
}

impl Default for SegmentOptions {
    fn default() -> Self {
        SegmentOptions {
            tone_type: ToneType::Symbol,
            mode: PinyinMode::Normal,
            surname: None,
            non_zh: NonZh::Spaced,
            non_zh_scope: None,
            v: VMode::Off,
            tone_sandhi: true,
            segmentit: TokenizationAlgorithm::MaxProbability,
            traditional: false,
            format: OutputFormat::AllSegment,
            separator: " ".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentPair {
    pub origin: String,
    pub result: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SegmentOutput {
    AllSegment(Vec<SegmentPair>),
    AllArray(Vec<Vec<SegmentPair>>),
    AllString(SegmentPair),
    PinyinSegment(Vec<String>),
    PinyinArray(Vec<Vec<String>>),
    PinyinString(String),
    ZhSegment(Vec<String>),
    ZhArray(Vec<Vec<String>>),
    ZhString(String),
}
