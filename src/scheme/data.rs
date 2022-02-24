use std::error::Error;

#[derive(Clone, Copy, Debug,PartialEq)]
pub enum MimeType {
    TextPlain,
    TextGemini,
    ImageJpeg,
    ImagePng,
    ImageSvg,
    Unknown,
}

#[derive(Clone,Debug,PartialEq)]
pub struct DataUrl {
    mime: MimeType,
    base64: bool,
    data: String,
}

#[derive(Clone,Debug,PartialEq)]
pub enum Data {
    Text(String),
    Bytes(Vec<u8>),
}

impl TryFrom<&str> for DataUrl {
    type Error = &'static str;

    fn try_from(url: &str) -> Result<Self, Self::Error> {
        let (scheme,remainder) = match url.split_once(":") {
            Some((s,r)) => (s,r),
            None => return Err("Malformed url"),
        };
        if scheme != "data" {
            return Err("Not a data url");
        }
        let (mime,data) = match remainder.split_once(",") {
            Some((m,d)) => (m,d),
            None => return Err("Malformed url"),
        };
        let base64 = mime.contains("base64");
        let mime = match mime.split_once(";") {
            Some((m,_)) => m,
            _ => mime,
        };
        let mimetype = match mime {
            "text/plain" => MimeType::TextPlain,
            "text/gemini" => MimeType::TextGemini,
            "image/jpeg" => MimeType::ImageJpeg,
            "image/png" => MimeType::ImagePng,
            "image/svg" => MimeType::ImageSvg,
            _ => MimeType::Unknown,
        };
        Ok(Self {
            mime: mimetype,
            base64,
            data: data.to_string(),
        })
    }
}

impl DataUrl {
    pub fn mime(&self) -> MimeType {
        self.mime
    }

    pub fn decode(&self) -> Result<Data, Box<dyn Error>> {
        match self.mime {
            MimeType::TextPlain | MimeType::TextGemini => {
                let pl = if self.base64 {
                    let tmp = base64::decode(&self.data)?;
                    String::from_utf8(tmp)?
                } else {
                    urlencoding::decode(&self.data)?.to_string()
                };
                Ok(Data::Text(pl))
            },
            MimeType::ImageJpeg | MimeType::ImagePng | MimeType::ImageSvg => {
                let pl = if self.base64 {
                    base64::decode(&self.data)?
                } else {
                    self.data.as_bytes().to_vec()
                };
                Ok(Data::Bytes(pl))
            },
            MimeType::Unknown => Err(String::from("Cannot decode unknown mimetype").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const B64_URL: &str = "data:text/plain;base64,R05VIGlzIG5vdCBVbml4Cg==";
    const PERCENT_URL: &str = "data:text/plain,GNU%20is%20not%20Unix";

    #[test]
    fn try_from_b64() {
        let dat = DataUrl::try_from(B64_URL).unwrap();
        assert_eq!(
            dat,
            DataUrl {
                mime: MimeType::TextPlain,
                base64: true,
                data: "R05VIGlzIG5vdCBVbml4Cg==".to_string(),
            }
        );
    }

    #[test]
    fn decode_b64() {
        let dat = DataUrl::try_from(B64_URL).unwrap();
        let out = dat.decode().unwrap();
        assert_eq!(out, Data::Text(String::from("GNU is not Unix\n")));
    }

    #[test]
    fn try_from_percent() {
        let dat = DataUrl::try_from(PERCENT_URL).unwrap();
        assert_eq!(
            dat,
            DataUrl {
                mime: MimeType::TextPlain,
                base64: false,
                data: "GNU%20is%20not%20Unix".to_string(),
            }
        );
    }

    #[test]
    fn decode_percent() {
        let dat = DataUrl::try_from(PERCENT_URL).unwrap();
        let out = dat.decode().unwrap();
        assert_eq!(out, Data::Text(String::from("GNU is not Unix")));
    }
}
