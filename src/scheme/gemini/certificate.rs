use {
    gtk::glib,
    lazy_static::lazy_static,
    std::{error, fmt, fs, io, path::PathBuf}
};

lazy_static! {
    static ref CERTDIR: PathBuf = certificate_dir();
}

fn certificate_dir() -> PathBuf {
    let mut dir = glib::user_data_dir();
    dir.push(env!("CARGO_PKG_NAME"));
    dir.push("certificates");
    dir
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Rcgen(rcgen::RcgenError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Rcgen(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rcgen::RcgenError> for Error {
    fn from(e: rcgen::RcgenError) -> Self {
        Self::Rcgen(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Rcgen(e) => Some(e),
        }
    }
}

pub fn generate(host: &str) -> Result<(), Error> {
    let mut f = CERTDIR.clone();
    f.push(&format!("{host}.pem"));
    let alt_names = vec![host.to_string(), "localhost".to_string()];
    let cert = rcgen::generate_simple_self_signed(alt_names)?;
    let pem = cert.serialize_pem()?;
    fs::write(f, pem)?;
    Ok(())
}
