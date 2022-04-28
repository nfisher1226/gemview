pub mod data;
pub mod file;
pub mod finger;
pub mod gemini;
pub mod gopher;

use gtk::{Label, pango::FontDescription};

#[derive(Clone, Debug)]
pub(crate) struct Content {
    pub url: Option<String>,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl Content {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            url: None,
            mime: tree_magic_mini::from_u8(&bytes).to_string(),
            bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Response {
    Success(Content),
    Error(String),
}

pub(crate) trait ToMarkup {
    fn to_markup(&self, _: &FontDescription) -> String;
}

pub(crate) trait ToLabel {
    fn to_label(&self, _: &FontDescription) -> Label;
}
