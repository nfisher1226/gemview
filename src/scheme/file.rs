use super::Content;
use std::convert::TryFrom;
use std::path::PathBuf;
use url::Url;

impl TryFrom<Url> for Content {
    type Error = &'static str;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        if url.scheme() != "file" {
            return Err("Error: not a file url");
        }
        let mut path = url.host_str().unwrap_or("").to_string();
        path.push_str(url.path());
        if path.is_empty() {
            return Err("Error: empty path");
        }
        let path = PathBuf::from(path);
        let path = if let Ok(p) = std::fs::canonicalize(&path) {
            p
        } else {
            path
        };
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.is_dir() {
                let gmi = path.to_gmi()?;
                Ok(Content {
                    mime: String::from("text/gemini"),
                    bytes: gmi.as_bytes().to_vec(),
                })
            } else if meta.is_file() {
                if let Ok(bytes) = std::fs::read(&path) {
                    let mut mime = tree_magic_mini::from_u8(&bytes).to_string();
                    if mime.starts_with("text/") {
                        mime = match &path.extension().map(|x| x.to_str()) {
                            Some(Some("gmi")) | Some(Some("gemini")) => String::from("text/gemini"),
                            _ => mime,
                        }
                    }
                    Ok(Content { mime, bytes })
                } else {
                    Err("Error reading file")
                }
            } else {
                Err("Error getting file type")
            }
        } else {
            Err("Error reading file metadata")
        }
    }
}

pub(crate) trait ToGmi {
    type Error;
    fn to_gmi(&self) -> Result<String, Self::Error>;
}

impl ToGmi for PathBuf {
    type Error = &'static str;

    fn to_gmi(&self) -> Result<String, Self::Error> {
        let mut page = format!("# Index of {}\n", &self.display());
        if let Some(parent) = self.parent() {
            let link = format!("=> file://{} parent directory\n\n", parent.display(),);
            page.push_str(&link);
        }
        if let Ok(entries) = std::fs::read_dir(self) {
            for entry in entries.flatten() {
                let link = format!(
                    "=> file://{} {}\n",
                    entry.path().display(),
                    entry.file_name().to_string_lossy(),
                );
                page.push_str(&link);
            }
            Ok(page)
        } else {
            Err("Error reading directory")
        }
    }
}
