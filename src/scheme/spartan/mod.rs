use super::ResponseParseError;

use {
    super::{Content, RequestError},
    std::{
        convert::TryFrom,
        error::Error,
        io::{Read, Write},
        net::ToSocketAddrs,
        time::Duration,
    },
    url::Url,
    urlencoding::decode,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Success,
    Redirect,
    ClientError,
    ServerError,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub status: Status,
    pub meta: String,
    pub data: Vec<u8>,
}

impl TryFrom<&[u8]> for Response {
    type Error = ResponseParseError;

    fn try_from(raw: &[u8]) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(ResponseParseError::EmptyResponse);
        }
        let lf = match raw.iter().enumerate().find(|(_i,b)| **b == b'\n') {
            Some((i,_)) => i,
            None => return Err(ResponseParseError::InvalidResponseHeader),
        };
        let header: &str = match std::str::from_utf8(&raw[..lf]) {
            Ok(s) => s,
            Err(_) => return Err(ResponseParseError::InvalidResponseHeader),
        };
        let (status, meta) = match header.split_once(' ') {
            None => return Err(ResponseParseError::InvalidResponseHeader),
            Some((s,m)) => (s, String::from(m.trim())),
        };
        let status = match status.parse::<u8>() {
            Ok(n) => match n {
                2 => Status::Success,
                3 => Status::Redirect,
                4 => Status::ClientError,
                5 => Status::ServerError,
                _ => return Err(ResponseParseError::InvalidResponseHeader),
            },
            Err(_) => return Err(ResponseParseError::InvalidResponseHeader),
        };
        let data = Vec::from(&raw[lf + 1..]);
        Ok(Response {
            status,
            meta,
            data,
        })
    }
}

pub(crate) fn request(url: &Url) -> Result<Content, Box<dyn Error>> {
    let host_str = match url.host_str() {
        Some(h) => format!("{}:{}", h, url.port().unwrap_or(300)),
        None => return Err(RequestError::DnsError.into()),
    };
    let mut it = host_str.to_socket_addrs()?;
    let socket_addrs = match it.next() {
        Some(s) => s,
        None => {
            let err = std::io::Error::new(std::io::ErrorKind::Other, "No data retrieved");
            return Err(err.into());
        }
    };
    match std::net::TcpStream::connect_timeout(&socket_addrs, Duration::new(10, 0)) {
        Err(e) => Err(e.into()),
        Ok(mut stream) => {
            let mut path = url.path().to_string();
            if let Some(q) = url.query() {
                path.push('?');
                path.push_str(q);
            }
            path.push_str(" 0\r\n");
            let path = decode(&path)?;
            stream.write_all(path.as_bytes()).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            Ok(Content::from_bytes(bytes))
        }
    }
}
