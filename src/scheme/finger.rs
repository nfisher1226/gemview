use url::Url;
use std::error::Error;
use std::io::{ Read, Write };
use std::net::ToSocketAddrs;
use std::time::Duration;

use super::Content;
use super::gemini::request::RequestError;

pub fn request(url: &Url) -> Result<Content, Box<dyn Error>> {
    let host_str = match url.host_str() {
        Some(h) => format!("{}:{}", h, url.port().unwrap_or(79)),
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
            let mut user = if url.username() == "" {
                match url.path() {
                    "" => "",
                    s => &s[1..],
                }
            } else {
                url.username()
            }.to_string();
            user.push_str("\r\n");
            stream.write_all(user.as_bytes()).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            Ok(Content::from_bytes(bytes))
        }
    }
}
