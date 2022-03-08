pub mod data;
pub mod file;
pub mod finger;
pub mod gemini;
pub mod gopher;

#[derive(Clone, Debug)]
pub(crate) struct Content {
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl Content {
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
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
