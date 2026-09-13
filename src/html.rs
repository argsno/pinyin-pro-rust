//! `html()` ported from `lib/core/html/index.ts`.

use crate::options::{HtmlOptions, PinyinMode, PinyinOptions, PinyinOutput, SurnameMode, TypeMode};
use crate::pinyin::pinyin;

/// Mirrors `html(text, options)`.
pub fn html(text: &str, options: HtmlOptions) -> String {
    let surname = options.surname.or_else(|| {
        Some(if options.mode == PinyinMode::Surname {
            SurnameMode::All
        } else {
            SurnameMode::Off
        })
    });
    let opts = PinyinOptions {
        type_mode: TypeMode::All,
        tone_type: options.tone_type,
        v: options.v.clone(),
        tone_sandhi: options.tone_sandhi,
        segmentit: options.segmentit,
        traditional: options.traditional,
        surname,
        mode: options.mode,
        ..Default::default()
    };
    let items = match pinyin(text, opts) {
        PinyinOutput::All(all) => all,
        _ => unreachable!("type all"),
    };
    let result_class = if options.result_class.is_empty() {
        "py-result-item".to_string()
    } else {
        options.result_class.clone()
    };
    let chinese_class = if options.chinese_class.is_empty() {
        "py-chinese-item".to_string()
    } else {
        options.chinese_class.clone()
    };
    let pinyin_class = if options.pinyin_class.is_empty() {
        "py-pinyin-item".to_string()
    } else {
        options.pinyin_class.clone()
    };
    let non_chinese_class = if options.non_chinese_class.is_empty() {
        "py-non-chinese-item".to_string()
    } else {
        options.non_chinese_class.clone()
    };
    items
    .iter()
    .map(|item| {
      let mut additional = String::new();
      for (classname, dict) in &options.custom_class_map {
        if dict.iter().any(|s| s == &item.origin) {
          additional.push(' ');
          additional.push_str(classname);
        }
      }
      if item.is_zh {
        let rp_open = if options.rp { "<rp>(</rp>" } else { "" };
        let rp_close = if options.rp { "<rp>)</rp>" } else { "" };
        format!(
          "<span class=\"{result_class}{additional}\"><ruby><span class=\"{chinese_class}\">{}</span>{rp_open}<rt class=\"{pinyin_class}\">{}</rt>{rp_close}</ruby></span>",
          item.origin, item.pinyin
        )
      } else if options.wrap_non_chinese {
        format!(
          "<span class=\"{non_chinese_class}{additional}\">{}</span>",
          item.origin
        )
      } else {
        item.origin.clone()
      }
    })
    .collect::<Vec<_>>()
    .join("")
}
