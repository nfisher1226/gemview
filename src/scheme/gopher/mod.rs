pub mod parser;
use {
    super::{Content, RequestError},
    parser::LineType,
    std::{
        error::Error,
        io::{Read, Write},
        net::ToSocketAddrs,
        time::Duration,
    },
    url::Url,
    urlencoding::decode,
};

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
                match &line[0..1] {
                    "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "+" | "g" | "I"
                    | "T" | ":" | ";" | "<" | "d" | "h" | "i" | "s" => continue,
                    _ => return false,
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
        Some(h) => format!("{h}:{}", url.port().unwrap_or(70)),
        None => return Err(RequestError::DnsError.into()),
    };
    let mut it = host_str.to_socket_addrs()?;
    let socket_addrs = if let Some(s) = it.next() {
        s
    } else {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "No data retrieved");
        return Err(err.into());
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
