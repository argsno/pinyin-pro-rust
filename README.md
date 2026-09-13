# pinyin_pro (Rust)

Rust port of [pinyin-pro](https://github.com/zh-lx/pinyin-pro) (`v3.29.4`).
Chinese-to-pinyin with word segmentation, tone sandhi, polyphones, surnames,
matching, HTML ruby output, segmentation detail, custom dicts and
traditional-character support.

中文文档：[`docs/`](docs/)（快速开始、API 参考、行为说明、FFI）。

## Layout

- `src/` — library (`pinyin`, `polyphonic`, `match_pinyin`, `convert`,
  `html`, `segment`, `custom_pinyin`, `add_dict`, …)
- `src/data/` — generated dictionary tables (from `packages/data` upstream)
- `tests/` — integration tests
- `examples/difftest.rs` — differential-test runner (JS parity harness)

## Use

```rust
use pinyin_pro::options::{PinyinOptions, PinyinOutput, ToneType};
use pinyin_pro::pinyin;

match pinyin("汉语拼音", PinyinOptions::default()) {
    PinyinOutput::Str(s) => println!("{s}"), // hàn yǔ pīn yīn
    _ => {}
}
```

## JS API mapping

| JS | Rust |
|---|---|
| `pinyin` | `pinyin::pinyin` |
| `polyphonic` | `polyphonic::polyphonic` |
| `match` | `match_::match_text` |
| `convert` | `convert::{convert, convert_array}` |
| `html` | `html::html` |
| `segment` (+`OutputFormat`) | `segment::{segment, }` + `options::OutputFormat` |
| `customPinyin` / `clearCustomDict` | `custom::{custom_pinyin, clear_custom_dict}` |
| `addDict` / `removeDict` | `dict_api::{add_dict, remove_dict}` |
| `addTraditionalDict` | `traditional::add_traditional_dict` |
| `getInitialAndFinal` / `getFinalParts` / `getNumOfTone` | `pinyin_utils::{initial_and_final, final_parts, num_of_tone}` |

Options are plain structs (`options::PinyinOptions`, …) with `Default`
matching the JS defaults. Like the JS singletons, custom dicts and added
dicts are process-global (`store`).

## Features

- `napi` — Node.js native addon (JSON in/out: `pinyin`, `match_text`, `convert`)
- `wasm` — WebAssembly bindings (`pinyin_wasm`, `match_wasm`, `convert_wasm`)

## Dev

```bash
cargo test
cargo clippy --all-targets
cargo check --features napi,wasm
```

## Parity notes

- Verified against the original TS implementation on 4221 differential ops
  (built-in dict, sampled + full `complete.json` dicts, custom/dict state
  sequences): zero mismatches.
- `match` returns Unicode scalar (char) indices. For BMP-only text these
  equal the JS UTF-16 indices; non-BMP characters count as one position
  instead of two.
- `non_zh_scope` takes a regex string (JS takes `RegExp`).
- Data tables are generated from the upstream `lib/data/*.ts` sources;
  single-char readings (`dict1`), word patterns (`dict2`–`dict5`),
  surnames and number rules are embedded at compile time. Regenerate with
  `node scripts/gen_data.mjs` (needs the upstream JS tree).
