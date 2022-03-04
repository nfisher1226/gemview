pub mod parser;
pub mod protocol;
pub mod request;

#[derive(Clone, Debug)]
pub struct Content {
    pub url: String,
    pub mime: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum Response {
    Success(Content),
    Error(String),
}
