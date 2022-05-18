pub mod data;
pub mod file;
pub mod finger;
pub mod gemini;
pub mod gopher;
pub mod spartan;

use gtk::{pango::FontDescription, Label};

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
pub struct Input {
    pub meta: String,
    pub url: String,
    pub sensitive: u8,
}

#[derive(Clone, Debug)]
pub(crate) enum Response {
    Success(Content),
    Redirect(String),
    RequestInput(Input),
    Error(String),
}

#[derive(Debug)]
/// A catch-all enum for any errors that may happen
/// while making and parsing the request
pub enum RequestError {
    /// Occurs when an [IO Error](std::io::Error) occurs.
    IoError(std::io::Error),
    /// Occurs when a DNS error occurs.
    DnsError,
    /// Occurs when some sort of [TLS error](native_tls::Error) occurs
    TlsError(String),
    //TlsError(rustls::Error),
    /// Occurs when the scheme given is unknown. Returns the scheme name.
    UnknownScheme(String),
    /// Occurs when the response from the server cannot be parsed.
    ResponseParseError(ResponseParseError),
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RequestError::IoError(e) => {
                write!(f, "IO error: {}", e)
            }
            RequestError::DnsError => {
                write!(f, "DNS Error")
            }
            RequestError::TlsError(e) => {
                write!(f, "TLS Error: {}", e)
            }
            RequestError::UnknownScheme(s) => {
                write!(f, "Unknown scheme {}", s)
            }
            RequestError::ResponseParseError(e) => {
                write!(f, "Response parse error: {}", e)
            }
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RequestError::IoError(e) => Some(e),
            RequestError::DnsError | RequestError::TlsError(_) | RequestError::UnknownScheme(_) => {
                None
            }
            RequestError::ResponseParseError(e) => Some(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An error in parsing a response header from a server
pub enum ResponseParseError {
    /// The entire response was empty.
    EmptyResponse,
    /// The response header was invalid and could not be parsed
    InvalidResponseHeader,
}

impl core::fmt::Display for ResponseParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ResponseParseError::EmptyResponse => {
                write!(f, "Error parsing response! The response was empty!")
            }
            ResponseParseError::InvalidResponseHeader => {
                write!(
                    f,
                    "Error parsing response! The response's header was invalid"
                )
            }
        }
    }
}

impl std::error::Error for ResponseParseError {}

pub(crate) trait ToMarkup {
    fn to_markup(&self, _: &FontDescription) -> String;
}

pub(crate) trait ToLabel {
    fn to_label(&self, _: &FontDescription) -> Label;
}
