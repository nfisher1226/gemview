use {
    super::{Content, RequestError},
    std::{
        error::Error,
        io::{Read, Write},
        net::ToSocketAddrs,
        time::Duration,
    },
    url::Url,
};

/// Make a finger protocol request
pub(crate) fn request(url: &Url) -> Result<Content, Box<dyn Error>> {
    let host_str = if let Some(h) = url.host_str() {
        format!("{h}:{}", url.port().unwrap_or(79))
    } else {
        return Err(RequestError::DnsError.into());
    };
    let mut it = host_str.to_socket_addrs()?;
    let Some(socket_addrs) = it.next() else {
        let err = std::io::Error::new(std::io::ErrorKind::Other, "No data retrieved");
        return Err(err.into());
    };
    match std::net::TcpStream::connect_timeout(&socket_addrs, Duration::new(10, 0)) {
        Err(e) => Err(e.into()),
        Ok(mut stream) => {
            let mut user = if url.username() == "" {
                match url.path() {
                    "" => "",
                    s => &s[1..],
                }
            } else {
                url.username()
            }
            .to_string();
            user.push_str("\r\n");
            stream.write_all(user.as_bytes()).unwrap();
            let mut bytes = vec![];
            stream.read_to_end(&mut bytes).unwrap();
            Ok(Content::from_bytes(bytes))
        }
    }
}
