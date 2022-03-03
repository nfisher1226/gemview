pub mod data;
pub mod finger;
pub mod gemini;
pub mod gopher;

#[derive(Clone, Debug)]
pub struct Content {
    pub mime: &'static str,
    pub bytes: Vec<u8>,
}

impl Content {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            mime: tree_magic_mini::from_u8(&bytes),
            bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Response {
    Success(Content),
    Error(String),
}
