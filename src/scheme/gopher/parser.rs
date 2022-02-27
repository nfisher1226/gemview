use gtk::pango::FontDescription;

#[derive(Clone, Debug, PartialEq)]
pub enum LineType {
    Text(String),
    Link(Link),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub display: String,
    pub path: String,
    pub host: String,
    pub port: String,
}

impl LineType {
    pub fn parse_line(line: &str) -> Option<Self> {
        if line == "." {
            return None;
        }
        if line.starts_with("i") {
            let mut text = line.split('\t').next().unwrap().to_string();
            text.remove(0);
            return Some(Self::Text(text));
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
    pub fn to_markup(&self, font: &FontDescription) -> String {
        format!(
            "<span font=\"{}\"><a href=\"gopher://{}:{}{}\">{}</a></span>\n",
            font.to_str(),
            &self.host,
            &self.port,
            &urlencoding::encode(&self.path),
            &self.display,
        )
    }
}
