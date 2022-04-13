use gtk::{glib, pango::FontDescription};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LineType {
    /// An ordinary text line
    Text(String),
    /// Gopher link
    Link(Link),
    /// Gopher query
    Query(Link),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Link {
    /// The string displayed to represent the link
    pub display: String,
    /// The path from the server root to this document
    pub path: String,
    /// This fqdn of the server
    pub host: String,
    /// The port this server runs on
    pub port: String,
}

impl LineType {
    pub(crate) fn parse_line(line: &str) -> Option<Self> {
        if line == "." {
            return None;
        }
        if line.starts_with('i') {
            let mut text = line.split('\t').next().unwrap().to_string();
            text.remove(0);
            Some(Self::Text(text))
        } else if line.starts_with('7') {
            Link::from_line(line).map(Self::Query)
        } else {
            let mut line = line.split('\t');
            let mut display = match line.next() {
                Some(d) => d.to_string(),
                None => return None,
            };
            let display = display.split_off(1);
            let path = match line.next() {
                Some(p) => p.to_string(),
                None => return None,
            };
            let host = match line.next() {
                Some(h) => h.to_string(),
                None => return None,
            };
            let port = match line.next() {
                Some(p) => p.to_string(),
                None => return None,
            };
            Some(Self::Link(Link {
                display,
                path,
                host,
                port,
            }))
        }
    }
}

impl Link {
    fn from_line(line: &str) -> Option<Self> {
        let mut els = line.split('\t');
        let mut display = match els.next() {
            Some(d) => d.to_string(),
            None => return None,
        };
        let display = display.split_off(1);
        let path = match els.next() {
            Some(p) => p.to_string(),
            None => return None,
        };
        let host = match els.next() {
            Some(h) => h.to_string(),
            None => return None,
        };
        let port = match els.next() {
            Some(p) => p.to_string(),
            None => return None,
        };
        Some(Self {
            display,
            path,
            host,
            port,
        })
    }

    /// Generates Pango markup from a Gopherr link
    pub(crate) fn to_markup(&self, font: &FontDescription) -> String {
        let link =
            format!("gopher://{}:{}{}", &self.host, &self.port, &self.path).replace(' ', "%20");
        format!(
            "<span font=\"{}\"><a href=\"{}\">{}</a></span>",
            font.to_str(),
            &link,
            glib::markup_escape_text(&self.display)
        )
    }
}
