//! `pinyin-pro` Rust port.
//!
//! Faithful port of [pinyin-pro](https://github.com/zh-lx/pinyin-pro):
//! Chinese-to-pinyin conversion with word segmentation, tone sandhi,
//! polyphones, surnames, matching, HTML ruby output, segmentation detail,
//! custom dictionaries and traditional-character support.
//!
//! The default API mirrors the JS functions with Rust-idiomatic option
//! structs; see [`pinyin()`] to start.

pub mod convert;
pub mod custom;
pub mod data;
pub mod dict_api;
pub mod html;
pub mod match_;
pub mod options;
pub mod pinyin;
pub mod pinyin_utils;
pub mod polyphonic;
pub mod segment;
pub mod segmentit;
pub mod store;
pub mod traditional;

#[cfg(feature = "napi")]
pub mod napi_bindings;
#[cfg(feature = "wasm")]
pub mod wasm_bindings;

// --- primary API (mirrors lib/index.ts) ---
pub use convert::{convert, convert_array};
pub use custom::{clear_custom_dict, custom_pinyin};
pub use dict_api::{add_dict, remove_dict};
pub use html::html;
pub use match_::match_text as match_pinyin;
pub use pinyin::{get_all_pinyin, get_pinyin, get_single_word_pinyin, pinyin};
pub use pinyin_utils::{final_parts, initial_and_final, num_of_tone};
pub use polyphonic::polyphonic;
pub use segment::segment;
pub use traditional::{add_traditional_dict, get_traditional_dict};
