use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UnknownIndex {
    pub word: String,
    pub start: i32,
    pub end: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WarningIndex {
    pub start: i32,
    pub end: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoundResponse {
    pub text: String,
    pub input_unknown_indexes: Vec<UnknownIndex>,
    pub output_unknown_indexes: Vec<UnknownIndex>,
    pub output_warning_indexes: Vec<WarningIndex>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnknownAttribute {
    pub uuid: String,
    pub unknown: Option<String>,
    pub num: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attributes {
    pub warning: Option<UnknownAttribute>,
    pub caution: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Srt {
    pub duration: f64,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Delta {
    pub insert: String,
    pub attributes: Option<Attributes>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoundDelta {
    pub input: Vec<Delta>,
    pub output: Vec<Delta>,
}
