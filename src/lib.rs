mod types;
mod utils;

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

use uuid::Uuid;

extern crate web_sys;
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

extern crate serde_json;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm!");
}

#[wasm_bindgen]
pub fn format_sound(input: &str, output: &str, response: JsValue) -> JsValue {
    let deserialized: types::SoundResponse = response.into_serde().unwrap();

    let mut input_delta: Vec<types::Delta> = Vec::new();
    let mut input_text = input.chars();
    let mut last_input_index = 0;
    for v in deserialized.input_unknown_indexes {
        let my_uuid = Uuid::new_v4();

        let mut delta = types::Delta {
            insert: (utils::text_iterator(&mut input_text, v.start - last_input_index)).to_string(),
            attributes: None,
        };
        input_delta.push(delta);
        delta = types::Delta {
            insert: (utils::text_iterator(&mut input_text, v.end - v.start)).to_string(),
            attributes: Some(types::Attributes {
                warning: Some(types::UnknownAttribute {
                    uuid: my_uuid.to_string(),
                    unknown: Some(v.word),
                    num: None,
                }),
                caution: None,
            }),
        };
        input_delta.push(delta);
        last_input_index = v.end;
    }

    let mut output_delta: Vec<types::Delta> = Vec::new();
    let mut output_text = output.chars();
    let mut last_output_index = 0;
    for v in deserialized.output_unknown_indexes {
        let my_uuid = Uuid::new_v4();

        let mut delta = types::Delta {
            insert: (utils::text_iterator(&mut output_text, v.start - last_output_index))
                .to_string(),
            attributes: None,
        };
        output_delta.push(delta);
        delta = types::Delta {
            insert: (utils::text_iterator(&mut output_text, v.end - v.start)).to_string(),
            attributes: Some(types::Attributes {
                warning: Some(types::UnknownAttribute {
                    uuid: my_uuid.to_string(),
                    unknown: Some(v.word),
                    num: None,
                }),
                caution: None,
            }),
        };
        output_delta.push(delta);
        last_output_index = v.end;
    }

    let response_delta = types::SoundDelta {
        input: input_delta,
        output: output_delta,
    };

    return JsValue::from_serde(&response_delta).unwrap();
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct types::SoundDelta {
//     input: Vec<types::Delta>,
//     output: Vec<types::Delta>,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct NewLineResponse {
    lines: Vec<String>,
    input: Vec<types::Delta>,
    errors: HashSet<String>,
    subtitle: String,
}

#[wasm_bindgen]
pub fn format_newline(response: JsValue, font: String, max_width: i32) -> JsValue {
    // utils::set_panic_hook();
    let deserialized: Vec<String> = response.into_serde().unwrap();

    let context = utils::initialize_canvas_context(font);

    let lines = utils::token_to_lines(&context, deserialized, max_width);

    let (partials, deltas, errors) = utils::line_to_sequence(&context, &lines, max_width);

    let subtitle = utils::generate_subtitle(partials);

    let return_value = NewLineResponse {
        lines: lines,
        input: deltas,
        errors: errors,
        subtitle: subtitle,
    };
    return JsValue::from_serde(&return_value).unwrap();
}
