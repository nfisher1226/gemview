//! Contents
//! ========
//! - [Introduction](#introduction)
//! - [Features](#features)
//! - [Usage](#usage)
//! ## Introduction
//! GemView is a [gemini protocol](https://gemini.circumlunar.space/) browser widget
//! for gtk+ (version 4) implemented in Rust.
//! ## Features
//! - [x] Browse and render gemini gemtext content
//! - [ ] Display plain text over gemini
//! - [ ] Browse and render gopher and plain text over gopher
//! - [ ] Display images served over gemini/gopher
//! - [ ] Open http(s) links in a *normal* browser
//! - [x] User customizable fonts
//! - [ ] User customizable colors
//! - [ ] Back/forward list
//! - [ ] History
//!
//! ## Usage
//! ```Yaml
//! [dependencies]
//! gemview = { git = "https://codeberg.org/jeang3nie/gemview" }
//!
//! [dependencies.gtk]
//! version = "~0.4"
//! package = "gtk4"
//! ```
//! ```Rust
//! use gemview::GemView;
//! use gtk::prelude::*;
//!
//! let browser = GemView::default();
//! let scroller = gtk::builders::ScrolledWindowBuilder::new()
//!     .child(&browser)
//!     .hexpand(true)
//!     .vexpand(true)
//!     .build();
//! let window = gtk::builders::WindowBuilder::new()
//!     .child(&scroller)
//!     .title("GemView")
//!     .build()
//! window.show();
//! browser.visit("gemini://gemini.circumlunar.space");
//! ```

use gmi::{gemtext, protocol, request};
use gmi::gemtext::GemtextNode;
use gmi::url::Url;
use glib::Object;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::pango::{FontDescription, Style, Weight};

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
    /// Returns the current uri being displayed
    pub fn uri(&self) -> String {
        let imp = self.imp();
        imp.uri.borrow().clone()
    }

    /// Sets the current uri
    fn set_uri(&self, uri: &str) {
        let imp = self.imp();
        *imp.uri.borrow_mut() = String::from(uri);
    }

    /// Returns the font used to render "normal" elements
    pub fn font_paragraph(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_paragraph.borrow().clone()
    }

    /// Sets the font used to render "normal" elements
    pub fn set_font_paragraph(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_paragraph.borrow_mut() = font;
    }

    /// Returns the font used to render "preformatted" elements
    pub fn font_pre(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_pre.borrow().clone()
    }

    /// Sets the font used to render "preformatted" elements
    pub fn set_font_pre(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_pre.borrow_mut() = font;
    }

    /// Returns the font used to render "blockquote" elements
    pub fn font_quote(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_quote.borrow().clone()
    }

    /// Sets the font used to render "blockquote" elements
    pub fn set_font_quote(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_quote.borrow_mut() = font;
    }

    /// Returns the font used to render H1 heading elements
    pub fn font_h1(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_h1.borrow().clone()
    }

    /// Sets the font used to render H1 heading elements
    pub fn set_font_h1(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_h1.borrow_mut() = font;
    }

    /// Returns the font used to render H2 heading elements
    pub fn font_h2(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_h2.borrow().clone()
    }

    /// Sets the font used to render H2 heading elements
    pub fn set_font_h2(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_h2.borrow_mut() = font;
    }

    /// Returns the font used to render H3 heading elements
    pub fn font_h3(&self) -> FontDescription {
        let imp = self.imp();
        imp.font_h3.borrow().clone()
    }

    /// Sets the font used to render H3 heading elements
    pub fn set_font_h3(&self, font: FontDescription) {
        let imp = self.imp();
        *imp.font_h3.borrow_mut() = font;
    }

    /// Renders the given `&str` as a gemtext document
    fn render_gmi(&self, data: &str) {
        self.clear();
        let buf = self.buffer();
        let mut iter;
        let nodes = gemtext::parse_gemtext(&data);
        for node in nodes {
            match node {
                GemtextNode::Text(text) => {
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            text,
                        ),
                    );
                },
                GemtextNode::Heading(text) => {
                    let font = self.font_h1();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            text,
                        ),
                    );
                },
                GemtextNode::SubHeading(text) => {
                    let font = self.font_h2();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            text,
                        ),
                    );
                },
                GemtextNode::SubSubHeading(text) => {
                    let font = self.font_h3();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            text,
                        ),
                    );
                },
                GemtextNode::ListItem(text) => {
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">{} {}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            "•",
                            text,
                        ),
                    );
                },
                GemtextNode::Link(link,text) => {
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    let fixed = link.replace("&", "&amp;");
                    let anchor = buf.create_child_anchor(&mut iter);
                    let label = gtk::builders::LabelBuilder::new()
                        .use_markup(true)
                        .tooltip_text(&fixed)
                        .label(&format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\"><a href=\"{}\">{}</a></span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            fixed,
                            match text {
                                Some(t) => t,
                                None => fixed.clone(),
                            },
                        )).build();
                    self.add_child_at_anchor(&label, &anchor);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                    let viewer = self.clone();
                    label.connect_activate_link(move |_,link| {
                        match viewer.visit(link) {
                            Err(e) => eprintln!("Error: {}", e),
                            _ => {},
                        };
                        gtk::Inhibit(true)
                    });
                },
                GemtextNode::Blockquote(text) => {
                    let font = self.font_quote();
                    iter = buf.end_iter();
                    let fixed = text.replace("&", "&amp;");
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span background=\"{}\" font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">  {}</span>\n",
                            "grey",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
                            fixed,
                        ),
                    );
                },
                GemtextNode::Preformatted(text,_) => {
                    let font = self.font_pre();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font_family=\"{}\" weight=\"{}\" size=\"{}pt\" style=\"{}\">  {}</span>\n",
                            match font.family() {
                                Some(f) => String::from(f),
                                None => String::from("Sans"),
                            },
                            font.weight().stringify(),
                            font.size(),
                            font.style().stringify(),
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

    /// Clears the text buffer
    pub fn clear(&self) {
        let buf = self.buffer();
        let (mut start, mut end) = buf.bounds();
        buf.delete(&mut start, &mut end);
    }

    fn absolute_url(&self, url: &str) -> String {
        if url.starts_with("gemini://") {
            String::from(url)
        } else if url.starts_with("//") {
            format!("gemini:{}", url)
        } else {
            format!("{}{}", self.uri(), url)
        }
    }

    /// Retrieves and then displays the given uri
    pub fn visit(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let abs = self.absolute_url(addr);
        let mut uri = Url::try_from(abs.as_str())?;
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
                    self.emit_by_name::<()>("page-loaded", &[&abs]);
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

trait Stringify {
    fn stringify(&self) -> String;
}

impl Stringify for Style {
    fn stringify(&self) -> String {
        String::from(match self {
            Style::Oblique => "oblique",
            Style::Italic => "italic",
            _ => "normal",
        })
    }
}

impl Stringify for Weight {
    fn stringify(&self) -> String {
        String::from(match self {
            Weight::Thin => "thin",
            Weight::Ultralight => "ultralight",
            Weight::Light => "light",
            Weight::Semilight => "semilight",
            Weight::Book => "book",
            Weight::Normal => "normal",
            Weight::Medium => "medium",
            Weight::Semibold => "semibold",
            Weight::Bold => "bold",
            Weight::Ultrabold => "bold",
            Weight::Heavy => "heavy",
            Weight::Ultraheavy => "ultraheavy",
            _ => "normal",
        })
    }
}
