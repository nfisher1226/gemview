use {
    crate::scheme::{ToLabel, ToMarkup},
    gtk::{gdk::Cursor, glib, pango::FontDescription, Label},
    std::fmt,
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum LineType {
    /// An ordinary text line
    Text(String),
    /// Gopher link
    Link(Link),
    /// Gopher query
    Query(Link),
    /// An http link
    Http(ExternLink),
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ExternLink {
    pub display: String,
    pub url: String,
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
        } else if line.starts_with('h') {
            let mut els = line.split('\t');
            let mut display = match els.next() {
                Some(d) => d.to_string(),
                None => return None,
            };
            let display = display.split_off(1);
            if let Some(next) = els.next() {
                if next.starts_with("URL:") {
                    if let Some((_, url)) = next.split_once(':') {
                        return Some(Self::Http(ExternLink::new(display, url.to_string())));
                    }
                }
            }
            Link::from_line(line).map(Self::Link)
        } else {
            Link::from_line(line).map(Self::Link)
        }
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gopher://{}:{}{}", &self.host, &self.port, &self.path,)
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
}

impl ToMarkup for Link {
    /// Generates Pango markup from a Gopher link
    fn to_markup(&self, font: &FontDescription) -> String {
        format!(
            "<span color=\"#00ff00\"> 🌐  </span><span font=\"{font}\"><a href=\"{}\">{}</a></span>",
            &self.to_string().replace(' ', "%20"),
            glib::markup_escape_text(&self.display)
        )
    }
}

impl ToLabel for Link {
    fn to_label(&self, font: &FontDescription) -> Label {
        gtk::builders::LabelBuilder::new()
            .use_markup(true)
            .tooltip_text(&self.to_string())
            .label(&self.to_markup(font))
            .cursor(&Cursor::from_name("pointer", None).unwrap())
            .build()
    }
}

impl ExternLink {
    pub(crate) fn new(display: String, url: String) -> Self {
        Self { display, url }
    }
}

impl ToMarkup for ExternLink {
    fn to_markup(&self, font: &FontDescription) -> String {
        format!(
            "<span color=\"#ff0000\"> 🌐  </span><span font=\"{font}\"><a href=\"{}\">{}</a></span>",
            &self.url,
            glib::markup_escape_text(&self.display)
        )
    }
}

impl ToLabel for ExternLink {
    fn to_label(&self, font: &FontDescription) -> Label {
        gtk::builders::LabelBuilder::new()
            .selectable(true)
            .use_markup(true)
            .tooltip_text(&self.url)
            .label(&self.to_markup(font))
            .cursor(&Cursor::from_name("pointer", None).unwrap())
            .build()
    }
}
