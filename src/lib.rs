use gmi::{gemtext, protocol, request};
use gmi::gemtext::GemtextNode;
use gmi::url::Url;
use glib::Object;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp;

glib::wrapper! {
    pub struct GemView(ObjectSubclass<imp::GemView>)
        @extends gtk::TextView, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
    }

impl GemView {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create `GemView`.")
    }

    pub fn with_label(label: &str) -> Self {
        Object::new(&[("label", &label)]).expect("Failed to create `GemView`.")
    }
}

impl Default for GemView {
    fn default() -> Self {
        Self::new()
    }
}

impl GemView {
    pub fn uri(&self) -> String {
        let imp = self.imp();
        imp.uri.borrow().clone()
    }

    fn set_uri(&self, uri: &str) {
        let imp = self.imp();
        *imp.uri.borrow_mut() = String::from(uri);
    }

    fn render_gmi(&self, data: &str) {
        let buf = self.buffer();
        let mut iter;
        let nodes = gemtext::parse_gemtext(&data);
        for node in nodes {
            match node {
                GemtextNode::Text(t) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            "Sans",
                            "light",
                            12,
                            "Normal",
                            t,
                        ),
                    );
                },
                GemtextNode::Heading(t) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            "Sans",
                            "ultrabold",
                            18,
                            "Normal",
                            t,
                        ),
                    );
                },
                GemtextNode::SubHeading(t) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            "Sans",
                            "bold",
                            16,
                            "Normal",
                            t,
                        ),
                    );
                },
                GemtextNode::SubSubHeading(t) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            "Sans",
                            "normal",
                            14,
                            "Normal",
                            t,
                        ),
                    );
                },
                GemtextNode::ListItem(t) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{} {}</span>\n",
                            "Sans",
                            "light",
                            12,
                            "Normal",
                            "•",
                            t,
                        ),
                    );
                },
                GemtextNode::Link(link,text) => {
                    iter = buf.end_iter();
                    let fixed = link.replace("&", "&amp;");
                    let anchor = buf.create_child_anchor(&mut iter);
                    let label = gtk::builders::LabelBuilder::new()
                        .use_markup(true)
                        .tooltip_text(&fixed)
                        .label(&format!(
                            "<a href=\"{}\">{}</a>",
                            fixed,
                            match text {
                                Some(t) => t,
                                None => fixed.clone(),
                            },
                        )).build();
                    self.add_child_at_anchor(&label, &anchor);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                },
                GemtextNode::Blockquote(text) => {
                    iter = buf.end_iter();
                    let fixed = text.replace("&", "&amp;");
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span background=\"{}\" font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">  {}</span>\n",
                            "grey",
                            "Sans",
                            "light",
                            12,
                            "Normal",
                            fixed,
                        ),
                    );
                },
                GemtextNode::Preformatted(text,_) => {
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">  {}</span>\n",
                            "Sans Mono",
                            "light",
                            12,
                            "Normal",
                            text,
                        ),
                    );
                },
                GemtextNode::EmptyLine => {
                    let mut iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                },
            }
        }
    }

    pub fn visit(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut uri = Url::try_from(addr)?;
        loop {
            let response = request::make_request(&uri)?;
            match response.status {
                protocol::StatusCode::Redirect(c) => {
                    println!("Redirect code {} with meta {}", c, response.meta);
                    uri = match Url::try_from(response.meta.as_str()) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("{}", e);
                            break;
                        },
                    };
                },
                protocol::StatusCode::Success(_) => {
                    let data = String::from_utf8_lossy(&response.data);
                    self.render_gmi(&data);
                    break;
                },
                s => {
                    eprintln!("Unknown status code: {:?}", s);
                    break;
                },
            }
        }
        self.set_uri(&uri.to_string());
        return Ok(())
    }
}
