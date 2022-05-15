use super::ResponseParseError;

use {
    super::RequestError,
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

impl TryFrom<u8> for Status {
    type Error = ResponseParseError;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            2 => Ok(Self::Success),
            3 => Ok(Self::Redirect),
            4 => Ok(Self::ClientError),
            5 => Ok(Self::ServerError),
            _ => Err(ResponseParseError::InvalidResponseHeader),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub status: Status,
    pub meta: String,
    pub data: Vec<u8>,
}

impl TryFrom<&Vec<u8>> for Response {
    type Error = ResponseParseError;

    fn try_from(raw: &Vec<u8>) -> Result<Self, Self::Error> {
        if raw.is_empty() {
            return Err(ResponseParseError::EmptyResponse);
        }
        let lf = match raw.iter().enumerate().find(|(_, b)| **b == b'\n') {
            Some((i, _)) => i,
            None => return Err(ResponseParseError::InvalidResponseHeader),
        };
        let header: &str = match std::str::from_utf8(&raw[..lf]) {
            Ok(s) => s,
            Err(_) => return Err(ResponseParseError::InvalidResponseHeader),
        };
        let (status, meta) = match header.split_once(' ') {
            None => return Err(ResponseParseError::InvalidResponseHeader),
            Some((s, m)) => (s, String::from(m.trim())),
        };
        let status = match status.parse::<u8>() {
            Ok(n) => Status::try_from(n)?,
            Err(_) => return Err(ResponseParseError::InvalidResponseHeader),
        };
        Ok(Response {
            status,
            meta,
            data: Vec::from(&raw[lf + 1..]),
        })
    }
}

impl Response {
    pub(crate) fn to_message(self, url: &mut Url) -> super::Response {
        match self.status {
            Status::Redirect => {
                println!("Redirect with meta {}", self.meta);
                url.set_path(&self.meta);
                super::Response::Redirect(url.to_string())
            }
            Status::Success => {
                let mime = if self.meta.starts_with("text/gemini") {
                    String::from("text/gemini")
                } else if let Some((mime, _)) = self.meta.split_once(' ') {
                    String::from(mime)
                } else {
                    self.meta
                };
                let url = Some(url.to_string());
                let content = super::Content {
                    url,
                    mime,
                    bytes: self.data,
                };
                super::Response::Success(content)
            }
            Status::ClientError => super::Response::Error(String::from("Client Error")),
            Status::ServerError => super::Response::Error(String::from("Client Error")),
        }
    }
}

pub(crate) fn request(url: &Url) -> Result<Response, Box<dyn Error>> {
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
            if path.is_empty() {
                path.push('/');
            }
            if let Some(q) = url.query() {
                path.push('?');
                path.push_str(q);
            }
            let path = decode(&path)?;
            let request = format!("{} {} 0\r\n", url.host_str().unwrap(), path);
            stream.write_all(request.as_bytes()).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            let response = Response::try_from(&bytes)?;
            Ok(response)
        }
    }
}

pub(crate) fn post(url: &Url, data: &[u8]) -> Result<Response, Box<dyn Error>> {
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
            let path = url.path().to_string();
            let path = decode(&path)?;
            let header = format!("{} {} {}", url.host_str().unwrap(), path, data.len());
            let request = [header.as_bytes(), data].concat();
            stream.write_all(&request).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            let response = Response::try_from(&bytes)?;
            Ok(response)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn status_from_u8() {
        assert_eq!(Status::try_from(2).unwrap(), Status::Success);
    }
    #[test]
    fn response_parse() {
        let raw = "2 text/gemini 0\r\nLorum Ipsum";
        let response = Response::try_from(raw.as_bytes()).unwrap();
        assert_eq!(response.status, Status::Success);
        assert_eq!(response.meta, "text/gemini 0");
        assert_eq!(response.data, "Lorum Ipsum".as_bytes());
    }
    #[test]
    fn response_parse_empty() {
        let raw = "";
        let response = Response::try_from(raw.as_bytes()).unwrap_err();
        assert_eq!(response, ResponseParseError::EmptyResponse);
    }
    #[test]
    fn response_parse_missing_space() {
        let raw = "2text/gemini\r\n#Hello!";
        let response = Response::try_from(raw.as_bytes()).unwrap_err();
        assert_eq!(response, ResponseParseError::InvalidResponseHeader);
    }
}
