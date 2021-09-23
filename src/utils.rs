use std::f64;

use std::collections::HashSet;
use std::str::Chars;
use wasm_bindgen::JsCast;

use web_sys::CanvasRenderingContext2d;

use crate::types;

extern crate web_sys;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn text_iterator(iter: &mut Chars, size: i32) -> String {
    let mut string = String::from("");

    for _ in 0..size {
        let c = iter.next();
        match c {
            Some(v) => string.push(v),
            None => break,
        }
    }
    return string;
}

pub fn initialize_canvas_context(font: String) -> CanvasRenderingContext2d {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.create_element("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    context.set_font(&font);
    return context;
}

fn measure_width(context: &CanvasRenderingContext2d, text: &str) -> f64 {
    let metrics = context.measure_text(text).unwrap();
    let width = metrics.width();
    return width;
}

pub fn token_to_lines(
    context: &CanvasRenderingContext2d,
    tokens: Vec<String>,
    max_width: i32,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut splitted: Vec<String> = Vec::new();
    let mut new_line_counter = 0;

    for token in tokens {
        if token == "" {
            if new_line_counter < 1 {
                if splitted.len() > 0 {
                    lines.push(splitted.join(""));
                    splitted.clear();
                }
            } else if new_line_counter < 2 {
                lines.push(String::new());
            }
            new_line_counter += 1;
            continue;
        }
        new_line_counter = 0;

        let width = measure_width(&context, &(splitted.join("") + &token));
        if width > max_width.into() {
            if splitted.len() > 0 {
                lines.push(splitted.join(""));
                splitted.clear();
                splitted.push(token);
            }
            continue;
        }
        if vec!["。", "！", "？", "-"]
            .iter()
            .any(|x| token.ends_with(x))
        {
            lines.push(splitted.join("") + &token);
            splitted.clear();
            continue;
        }
        splitted.push(token);
    }
    if splitted.len() > 0 {
        lines.push(splitted.join(""));
    }
    return lines;
}

fn calc_srt(text1: String, text2: Option<&str>) -> types::Srt {
    let base_time = 7.5 as f64;
    let max_length = 30 as f64;
    let duration: f64;
    let text: String;

    match text2 {
        Some(_text2) => {
            let size = (text1.len() + _text2.len()) as f64;
            duration = ((size / max_length) as f64 * base_time).ceil();
            text = format!("<b>{}</b>\n<b>{}</b>", text1, _text2);

            return types::Srt {
                duration: duration,
                text: text,
            };
        }
        None => {
            if text1 == "☆" {
                duration = 2 as f64;
            } else {
                let size = text1.len() as f64;
                let ceil = ((size / max_length) * base_time).ceil();
                let max = ceil.max(2 as f64);
                let min = max.min(15 as f64);
                duration = min;
            }
            text = format!("<b>{}</b>", text1);
            return types::Srt {
                duration: duration,
                text: text,
            };
        }
    }
}

pub fn line_to_sequence(
    context: &CanvasRenderingContext2d,
    lines: &Vec<String>,
    max_width: i32,
) -> (Vec<types::Srt>, Vec<types::Delta>, HashSet<String>) {
    let mut partials: Vec<types::Srt> = Vec::new();
    let mut last: Option<String> = None;
    let mut seq_counter = 0;
    let mut errors: HashSet<String> = HashSet::new();
    let mut deltas: Vec<types::Delta> = Vec::new();

    for line in lines.iter() {
        let text = line.trim();
        let width = measure_width(context, text);
        let mut caution: Option<String> = None;

        if width > max_width.into() {
            caution = Some("too long sentence, cannot be splitted".to_string());
            errors.insert("Sentences continue for more than three lines.".to_string());
        }

        seq_counter += 1;
        if text == "" || text == "。" {
            seq_counter = 0;
            match last {
                Some(v) => {
                    partials.push(calc_srt(v, None));
                    last = None;
                }
                None => {}
            }
        } else {
            if seq_counter > 2 {
                deltas.push(types::Delta {
                    insert: "\n".to_string(),
                    attributes: None,
                });
                seq_counter = 0;
            }
            match last {
                Some(v) => {
                    partials.push(calc_srt(v, Some(&text)));
                    last = None;
                }
                None => {
                    last = Some(text.to_string());
                }
            }
        }

        match caution {
            Some(v) => deltas.push(types::Delta {
                insert: format!("{}{}", text, "\n"),
                attributes: Some(types::Attributes {
                    caution: Some(v),
                    warning: None,
                }),
            }),
            None => deltas.push(types::Delta {
                insert: format!("{}{}", text, "\n"),
                attributes: None,
            }),
        }
    }
    match last {
        Some(v) => partials.push(calc_srt(v, None)),
        None => {}
    }

    return (partials, deltas, errors);
}

struct SubtitleTime {
    hour: String,
    minute: String,
    second: String,
}

fn calc_subtitle_time(second_s: f64) -> SubtitleTime {
    let hour = (second_s / 3600.0).floor() as i32;
    let minute = ((second_s - hour as f64) * 60.0 / 60.0).floor() as i32;
    let second = (second_s % 60.0) as i32;

    let hour_s = format!("00{}", (hour + 1).to_string());
    let minute_s = format!("00{}", minute.to_string());
    let second_s = format!("00{}", second.to_string());

    return SubtitleTime {
        hour: (&hour_s[hour_s.len() - 2..]).to_string(),
        minute: (&minute_s[minute_s.len() - 2..]).to_string(),
        second: (&second_s[second_s.len() - 2..]).to_string(),
    };
}

pub fn generate_subtitle(partials: Vec<types::Srt>) -> String {
    let mut result = String::new();
    let mut last_time: Option<f64> = None;
    for (i, srt) in partials.iter().enumerate() {
        let start: f64;
        match last_time {
            Some(v) => start = v,
            None => start = 0.0,
        }
        let end = start + srt.duration;

        let start_s = calc_subtitle_time(start);
        let end_s = calc_subtitle_time(end);
        let index = i + 1;
        let block = format!(
            "{}\n{}:{}:{},000 --> {}:{}:{},000\n{}\n\n",
            index.to_string(),
            start_s.hour,
            start_s.minute,
            start_s.second,
            end_s.hour,
            end_s.minute,
            end_s.second,
            srt.text,
        );
        result.push_str(&block);
        last_time = Some(end);
    }
    return result;
}
