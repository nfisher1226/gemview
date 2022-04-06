use super::gemini::request::RequestError;
use std::error::Error;
use std::io::{Read, Write};
use std::net::ToSocketAddrs;
use std::time::Duration;
use url::Url;
use urlencoding::decode;

pub mod parser;
use super::Content;
use parser::LineType;

pub(crate) trait GopherMap {
    /// Validates that self is a valid Gopher map
    fn is_map(&self) -> bool;

    fn parse(&self) -> Vec<LineType>;
}

impl GopherMap for Content {
    fn is_map(&self) -> bool {
        if self.mime.starts_with("text") {
            let page = String::from_utf8_lossy(&self.bytes);
            for line in page.lines() {
                if line == "." {
                    break;
                }
                if !line.starts_with('0')
                    && !line.starts_with('1')
                    && !line.starts_with('2')
                    && !line.starts_with('3')
                    && !line.starts_with('4')
                    && !line.starts_with('5')
                    && !line.starts_with('6')
                    && !line.starts_with('7')
                    && !line.starts_with('8')
                    && !line.starts_with('9')
                    && !line.starts_with('+')
                    && !line.starts_with('g')
                    && !line.starts_with('I')
                    && !line.starts_with('T')
                    && !line.starts_with(':')
                    && !line.starts_with(';')
                    && !line.starts_with('<')
                    && !line.starts_with('d')
                    && !line.starts_with('h')
                    && !line.starts_with('i')
                    && !line.starts_with('s')
                {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn parse(&self) -> Vec<LineType> {
        let mut ret = vec![];
        for line in String::from_utf8_lossy(&self.bytes).lines() {
            if let Some(line) = LineType::parse_line(line) {
                ret.push(line);
            }
        }
        ret
    }
}

fn trim_path(path: String) -> String {
    if path.starts_with("/0/")
        || path.starts_with("/1/")
        || path.starts_with("/g/")
        || path.starts_with("/I/")
        || path.starts_with("/9/")
    {
        path[2..].to_string()
    } else {
        path
    }
}

pub(crate) fn request(url: &Url) -> Result<Content, Box<dyn Error>> {
    let host_str = match url.host_str() {
        Some(h) => format!("{}:{}", h, url.port().unwrap_or(70)),
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
            let mut path = trim_path(path);
            if let Some(q) = url.query() {
                path.push('?');
                path.push_str(q);
            }
            path.push_str("\r\n");
            let path = decode(&path)?;
            stream.write_all(path.as_bytes()).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            Ok(Content::from_bytes(bytes))
        }
    }
}
