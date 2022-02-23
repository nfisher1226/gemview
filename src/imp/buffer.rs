use gmi::protocol::Response;
use std::error::Error;

#[derive(Clone, Debug, Default)]
pub struct Buffer {
    pub mime: String,
    pub content: Vec<u8>,
}

impl Buffer {
    pub fn from_gmi_response(response: Response) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mime: response.meta.clone(),
            content: response.data.clone(),
        })
    }
}
