pub mod parser;
pub mod protocol;
pub mod request;

#[derive(Clone, Debug)]
pub struct Input {
    pub meta: String,
    pub url: String,
    pub sensitive: u8,
}

#[derive(Clone, Debug)]
pub(crate) enum Response {
    Success(super::Content),
    RequestInput(Input),
    Error(String),
}
