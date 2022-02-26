use urlencoding::decode;
use url::Url;
use gmi::request::RequestError;
use std::error::Error;
use std::io::{ Read, Write };
use std::net::ToSocketAddrs;
use std::time::Duration;

pub mod parser;
use parser::LineType;

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

    pub fn is_map(&self) -> bool {
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

    pub fn parse(&self) -> Vec<LineType> {
        let mut ret = vec![];
        for line in String::from_utf8_lossy(&self.bytes).lines() {
            if let Some(line) = LineType::parse_line(line) {
                ret.push(line);
            }
        }
        ret
    }
}

pub fn request(url: &Url) -> Result<Content, Box<dyn Error>> {
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
        },
    };
    match std::net::TcpStream::connect_timeout(
        &socket_addrs,
        Duration::new(10, 0),
    ) {
        Err(e) => return Err(e.into()),
        Ok(mut stream) => {
            let mut path = url.path().to_string();
            if let Some(q) = url.query() {
                path.push_str("?");
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
