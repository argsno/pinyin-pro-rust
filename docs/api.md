# API 参考

## `pinyin` — 汉字转拼音

```rust
pub fn pinyin(text: &str, options: PinyinOptions) -> PinyinOutput
```

空串返回空（`Str("")` / `Arr([])` / `All([])`）。

### `PinyinOptions`

| 字段 | 类型 | 默认 | 说明 |
|---|---|---|---|
| `pattern` | `PatternKind` | `Pinyin` | 返回内容：`Pinyin` 完整拼音，`Initial` 声母，`Final` 韵母，`Num` 声调数字，`First` 首字母，`FinalHead` 韵头，`FinalBody` 韵腹，`FinalTail` 韵尾 |
| `tone_type` | `ToneType` | `Symbol` | `Symbol` 符号（hàn），`Num` 数字（han4），`None` 无调（han）。`pattern=Num` 时强制 `None` |
| `type_mode` | `TypeMode` | `Str` | `Str` 空格连接；`Array` 数组；`All` 详情数组（此时 `pattern` 强制 `Pinyin`） |
| `multiple` | `bool` | `false` | 仅单字生效：返回全部读音，如 `好` → `hǎo hào`；去调后同音去重 |
| `mode` | `PinyinMode` | `Normal` | `Surname` 为姓氏模式（等价于 `surname=All`） |
| `surname` | `Option<SurnameMode>` | `None` | `None` 时按 `mode` 推导。`All` 全文，`Head` 仅开头，`Off` 关闭 |
| `tone_sandhi` | `bool` | `true` | 一/不变调；`false` 时恢复 `yī` / `bù` |
| `segmentit` | `TokenizationAlgorithm` | `MaxProbability` | `ReverseMaxMatch`(1) 逆向最大匹配，`MaxProbability`(2) 最大概率，`MinTokenization`(3) 最少分词 |
| `non_zh` | `NonZh` | `Spaced` | 非汉字处理：`Spaced` 逐字保留，`Consecutive` 相连合并，`Removed` 移除 |
| `non_zh_scope` | `Option<String>` | `None` | 正则字符串；只有命中的非汉字字符才受 `non_zh` 处理 |
| `remove_non_zh` | `bool` | `false` | 等价于 `non_zh=Removed` |
| `v` | `VMode` | `Off` | `ü` 的写法（仅去调后生效）：`V` → `v`，`Custom(s)` → `s` |
| `separator` | `String` | `" "` | `Str` 模式的连接符 |
| `initial_pattern` | `InitialPattern` | `Yw` | `Yw` 把 y/w 视为声母；`Standard` 则声母为空 |
| `traditional` | `bool` | `false` | 繁体模式（配合 `add_traditional_dict`） |

示例：

```rust
// 好 → hǎo hào；非单字时 multiple 无效
// 万俟 + Surname 模式 → mò qí
// 一个 → yí gè；说一说 → shuō yi shuō（叠词轻声）
// 不是 + tone_sandhi:false → bù shì
```

### `AllInfo`（`type_mode=All`）

`origin` 原字，`pinyin` 拼音，`initial` 声母，`final_` 韵母（字段名 `final_`，
对应 `final`），`num` 声调数字，`first` 首字母，`final_head/body/tail`，
`is_zh` 是否汉字，`polyphonic` 全部读音（当前排首），`in_zh_range` 是否在字典内，
`result` 最终结果（与 `pinyin` 相同）。

## `polyphonic` — 多音字

```rust
pub fn polyphonic(text: &str, options: PolyphonicOptions) -> PolyphonicOutput
```

每个字返回全部读音。空串返回 `Arr([])`。

```rust
// polyphonic("好好学习", 默认) → Str(["hǎo hào", "hǎo hào", "xué", "xí"])
// type array → Arr([[hǎo, hào], [hǎo, hào], [xué], [xí]])
// type all   → All(PolyInfo 二维数组，与 AllInfo 同字段，无 polyphonic)
```

`PolyphonicOptions` 字段：`pattern` / `tone_type` / `type_mode` / `non_zh` /
`non_zh_scope` / `remove_non_zh` / `v` / `initial_pattern`，含义同 `PinyinOptions`。

## `match_pinyin` — 拼音匹配

```rust
pub fn match_pinyin(text: &str, query: &str, options: MatchOptions) -> Option<Vec<usize>>
```

命中返回原文字符下标数组，失败返回 `None`。

```rust
// match_pinyin("欢迎使用汉语拼音", "hy", 默认) → Some([0, 1])
// match_pinyin("汉语拼音", "hanpin", 默认)     → Some([0, 2])
// match_pinyin("开会", "kaig", 默认)           → None
```

### `MatchOptions`

| 字段 | 默认 | 说明 |
|---|---|---|
| `precision` | `First` | 单字匹配精度：`First` 首字母，`Start` 开头，`Every` 完整拼音（仅影响中间字；`Any` 走宽松整串匹配） |
| `continuous` | `false` | 命中下标必须连续 |
| `space` | `Ignore` | 查询串空格：`Ignore` 忽略，`Preserve` 保留 |
| `last_precision` | `Start` | 最后一个字的精度（`query` 尾部 ≤6 字符时用它收尾） |
| `insensitive` | `true` | 大小写不敏感 |
| `v` | `Off` | `ü`→`v` 或自定义字符后匹配 |

注意：下标是字符（Unicode scalar）序号。纯 BMP 文本与原 JS 的 UTF-16 序号一致；
含非 BMP 字符（如 𠄼）时，每个 such 字符只占 1 位（JS 占 2 位）。

声调只在字典侧剥离：`match_pinyin("女", "nü")` 命中，`"nv"` 不命中（除非 `v=V`）；
查询串里的声调符号按原样匹配。

## `convert` — 拼音格式互转

```rust
pub fn convert(text: &str, options: ConvertOptions) -> String
pub fn convert_array(items: Vec<String>, options: ConvertOptions) -> Vec<String>
```

```rust
// convert("pin1 yin1", 默认)                  → "pīn yīn"
// convert("pīn yīn", {SymbolToNum})           → "pin1 yin1"
// convert("pīn yīn", {ToneNone})              → "pin yin"
// convert("dian3r", 默认)                     → "diǎnr"（儿化）
// convert("zhong1-guo2", {separator:"-"})     → "zhōng-guó"
```

`ConvertOptions`：`separator`（默认 `" "`，分割与连接都用它），
`format`：`NumToSymbol` / `SymbolToNum` / `ToneNone`。

## `html` — 注音 HTML

```rust
pub fn html(text: &str, options: HtmlOptions) -> String
```

```rust
// html("汉", 默认) →
// <span class="py-result-item"><ruby><span class="py-chinese-item">汉</span><rp>(</rp><rt class="py-pinyin-item">hàn</rt><rp>)</rp></ruby></span>
// html("a", 默认) → "a"（非汉字原样返回）
```

`HtmlOptions` 默认：`result_class="py-result-item"`，
`chinese_class="py-chinese-item"`，`pinyin_class="py-pinyin-item"`，
`non_chinese_class="py-non-chinese-item"`，`wrap_non_chinese=false`（非汉字不包 span），
`tone_type=Symbol`，`tone_sandhi=true`，`rp=true`（保留 `<rp>(</rp>`），`v=Off`，
`traditional=false`，`custom_class_map=[]`（`[(类名, [字])]`，命中则追加类名）。

## `segment` — 分词

```rust
pub fn segment(text: &str, options: SegmentOptions) -> SegmentOutput
```

`SegmentOptions`：`tone_type` / `mode` / `surname` / `non_zh` / `non_zh_scope` /
`v` / `tone_sandhi` / `segmentit` / `traditional`（同 pinyin），外加
`format: OutputFormat`（默认 `AllSegment`）、`separator`（默认 `" "`）。

`OutputFormat`：`AllSegment(1)` `[{origin, result}]`，
`AllArray(2)` 分组详情，`AllString(3)` `{origin, result}`，
`PinyinSegment(4)` / `PinyinArray(5)` / `PinyinString(6)`，
`ZhSegment(7)` / `ZhArray(8)` / `ZhString(9)`。

```rust
// segment("我叫令狐冲", 默认) → AllSegment([我/叫/令狐/冲])
//   其中令狐一段 result 为 "línghú"（段内无空格连接）
// segment("汉语拼音", {PinyinString}) → "hàn yǔ pīn yīn"
```

## `custom_pinyin` / `clear_custom_dict` — 自定义拼音

```rust
pub fn custom_pinyin(config: &[(String, String)], options: CustomPinyinOptions)
pub fn clear_custom_dict(kinds: &[CustomDictKind])
```

```rust
// custom_pinyin(&[("哈什玛".into(), "hà shén mǎ".into())], 默认)
// pinyin("哈什玛") → "hà shén mǎ"（自定义词优先，概率最高）
// clear_custom_dict(&[CustomDictKind::Pinyin])
```

`CustomPinyinOptions`：`multiple` / `polyphonic` 为 `Option<CustomHandle>`。
`Add` 把按字拆出的读音追加到 `multiple`/`polyphonic` 读音表（如
`{"你好": "mi kao"}` 使 `你` 多出 `mi`、`好` 多出 `kao`）；
`Replace` 直接覆盖。`CustomDictKind`：`Pinyin` / `Multiple` / `Polyphonic`。

## `add_dict` / `remove_dict` — 词典

```rust
pub fn add_dict(dict: &[(String, DictValue)], options: AddDictOptions)
pub fn remove_dict(name: Option<&str>)
```

```rust
// add_dict(&[("新词X".into(), "xīn cí".into())], 默认)
// pinyin("这是新词X和测试词") → "zhè shì xīn cí  hé cè shì cí"
//   （X 无读音占空位，与 JS 一致）
// remove_dict(None) 后 X 恢复为原样透传
```

`DictValue { pinyin, probability: Option<f64>, pos: Option<String>, is_array: bool }`，
`&str` 可 `into()` 为纯拼音词条。单字词条默认追加读音（`Dict1Handle::Add`），
`Replace` 覆盖，`Ignore` 只加分词不碰单字表。`remove_dict` 移除同名（默认）词条并恢复单字读音。

注意：拼音数与字数不一致时，多出/缺失的位置按空串处理（与 JS 一致）。

## `add_traditional_dict` — 繁体

```rust
pub fn add_traditional_dict(dict: &[(String, String)])
pub fn get_traditional_dict() -> HashMap<char, char>
```

`add_traditional_dict(&[("國".into(), "国".into())])` 后，
`pinyin("中國人", {traditional: true})` → `"zhōng guó rén"`。

## `pinyin_utils` — 纯函数

`strip_tone`（去调）、`num_of_tone`（取调号，如 `"hàn yǔ"`→`"4 3"`）、
`pinyin_with_num`（符号转数字）、`first_letter`（首字母）、
`initial_and_final`（声母韵母，j/q/x 遇 u 转 ü，如 `xué`→`("x", "üé")`）、
`final_parts` / `final_parts_from_final`（韵头/韵腹/韵尾）。
