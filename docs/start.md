# 快速开始

`Cargo.toml`：

```toml
[dependencies]
pinyin_pro = { git = "https://github.com/argsno/pinyin-pro" }
```

最小示例：

```rust
use pinyin_pro::options::{PinyinOptions, PinyinOutput};
use pinyin_pro::pinyin;

match pinyin("汉语拼音", PinyinOptions::default()) {
    PinyinOutput::Str(s) => println!("{s}"), // hàn yǔ pīn yīn
    _ => {}
}
```

常用一行式：

```rust
use pinyin_pro::options::{PinyinOptions, PinyinOutput, ToneType};

// 去声调
let out = pinyin_pro::pinyin(
    "汉语拼音",
    PinyinOptions { tone_type: ToneType::None, ..Default::default() },
);
assert!(matches!(out, PinyinOutput::Str(ref s) if s == "han yu pin yin"));

// 数组返回
let out = pinyin_pro::pinyin(
    "汉语拼音",
    PinyinOptions { type_mode: pinyin_pro::options::TypeMode::Array, ..Default::default() },
);
assert!(matches!(out, PinyinOutput::Arr(ref v) if v == &["hàn", "yǔ", "pīn", "yīn"]));
```

下一步：[API 参考](api.md)。
