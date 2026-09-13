# FFI（Node / WASM）

## Node（napi）

```bash
cargo build --features napi
```

按 napi-rs 流程生成 `.node` 文件（需 `@napi-rs/cli` 打包，见其文档）。
三个函数，选项一律走 JSON 字符串，结果走 JSON 字符串：

- `pinyin(text, optionsJson?) -> string`
- `match_text(text, query, optionsJson?) -> string`（JSON 数组或 `null`）
- `convert(text, optionsJson?) -> string`

选项键名与原 JS 包一致（小驼峰）：

```js
pinyin('汉语拼音', JSON.stringify({ toneType: 'none', type: 'array' }))
// '["han","yu","pin","yin"]'
```

支持的键：`pattern`、`toneType`、`type`、`multiple`、`mode`、`surname`、
`toneSandhi`、`segmentit`、`nonZh`、`removeNonZh`、`v`、`separator`、
`initialPattern`、`traditional`；`match` 另支持 `precision`、`continuous`、
`space`、`lastPrecision`、`insensitive`；`convert` 支持 `separator`、`format`。

## WASM（wasm-bindgen）

```bash
cargo build --target wasm32-unknown-unknown --features wasm
```

- `pinyin_wasm(text, optionsJson?) -> string`
- `match_wasm(text, query) -> string`
- `convert_wasm(text, format?) -> string`

选项与返回值同 napi（JSON 字符串进出），`match` 用默认选项。
