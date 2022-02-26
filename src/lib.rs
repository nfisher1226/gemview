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
//! - [x] Display plain text over gemini
//! - [x] Display images over gemini
//! - [x] Display text and images from `data://` url's
//! - [ ] Browse and render gopher and plain text over gopher
//! - [x] Open http(s) links in a *normal* browser
//! - [x] User customizable fonts
//! - [x] User customizable colors (via CSS)
//! - [x] Back/forward list
//! - [ ] History
//!
//! ## Usage
//! ```Yaml
//! [dependencies]
//! gemview = 0.1.0
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

use glib::Object;
use gmi::gemtext::GemtextNode;
use gmi::url::Url;
use gmi::{gemtext, protocol, request};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{Cancellable, MemoryInputStream, Menu, MenuItem};
use gtk::glib;
use gtk::pango::FontDescription;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use textwrap::fill;

use std::error::Error;

mod scheme;
use scheme::data::{Data, DataUrl, MimeType};
use scheme::gopher;
mod imp;

glib::wrapper! {
/// The gemini browser widget is a subclass of the `TextView` widget which
/// has been customized for browsing [geminispace](https://gemini.circumlunar.space).
pub struct GemView(ObjectSubclass<imp::GemView>)
    @extends gtk::TextView, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable, gtk::Actionable;
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
    /// Returns the current uri
    pub fn uri(&self) -> String {
        self.imp().history.borrow().uri.clone()
    }

    /// Sets the current uri
    pub fn set_uri(&self, uri: &str) {
        self.imp().history.borrow_mut().uri = String::from(uri);
    }

    fn previous(&self) -> Option<String> {
        self.imp().history.borrow_mut().previous()
    }

    /// Returns `true` if there are any items in the `back` history list
    pub fn has_previous(&self) -> bool {
        self.imp().history.borrow().has_previous()
    }

    /// If there are any items in the `back` history list, retrieves the most
    /// recent one and visits that uri
    ///
    /// ## Errors
    /// Propagates any page load errors
    pub fn go_previous(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(uri) = self.previous() {
            _ = self.load(&uri)?;
            self.emit_by_name::<()>("page-loaded", &[&uri]);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn next(&self) -> Option<String> {
        let imp = self.imp();
        imp.history.borrow_mut().next()
    }

    /// Returns `true` if there are any items in the `forward` history list
    pub fn has_next(&self) -> bool {
        self.imp().history.borrow().has_next()
    }

    /// If there are any items in the `forward` history list, retrieves the most
    /// recent item and visits that uri
    ///
    /// ## Errors
    /// Propagates any page load errors
    pub fn go_next(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(uri) = self.next() {
            _ = self.load(&uri)?;
            self.emit_by_name::<()>("page-loaded", &[&uri]);
            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn append_history(&self, uri: &str) {
        self.imp().history.borrow_mut().append(uri.to_string());
    }

    pub fn buffer_mime(&self) -> String {
        self.imp().buffer.borrow().mime.clone()
    }

    pub fn set_buffer_mime(&self, mime: &str) {
        self.imp().buffer.borrow_mut().mime = mime.to_string();
    }

    pub fn buffer_content(&self) -> Vec<u8> {
        self.imp().buffer.borrow().content.clone()
    }

    pub fn set_buffer_content(&self, content: &[u8]) {
        self.imp().buffer.borrow_mut().content = content.to_vec();
    }

    /// Returns the font used to render "normal" elements
    pub fn font_paragraph(&self) -> FontDescription {
        self.imp().font_paragraph.borrow().clone()
    }

    /// Sets the font used to render "normal" elements
    pub fn set_font_paragraph(&self, font: FontDescription) {
        *self.imp().font_paragraph.borrow_mut() = font;
    }

    /// Returns the font used to render "preformatted" elements
    pub fn font_pre(&self) -> FontDescription {
        self.imp().font_pre.borrow().clone()
    }

    /// Sets the font used to render "preformatted" elements
    pub fn set_font_pre(&self, font: FontDescription) {
        *self.imp().font_pre.borrow_mut() = font;
    }

    /// Returns the font used to render "blockquote" elements
    pub fn font_quote(&self) -> FontDescription {
        self.imp().font_quote.borrow().clone()
    }

    /// Sets the font used to render "blockquote" elements
    pub fn set_font_quote(&self, font: FontDescription) {
        *self.imp().font_quote.borrow_mut() = font;
    }

    /// Returns the font used to render H1 heading elements
    pub fn font_h1(&self) -> FontDescription {
        self.imp().font_h1.borrow().clone()
    }

    /// Sets the font used to render H1 heading elements
    pub fn set_font_h1(&self, font: FontDescription) {
        *self.imp().font_h1.borrow_mut() = font;
    }

    /// Returns the font used to render H2 heading elements
    pub fn font_h2(&self) -> FontDescription {
        self.imp().font_h2.borrow().clone()
    }

    /// Sets the font used to render H2 heading elements
    pub fn set_font_h2(&self, font: FontDescription) {
        *self.imp().font_h2.borrow_mut() = font;
    }

    /// Returns the font used to render H3 heading elements
    pub fn font_h3(&self) -> FontDescription {
        self.imp().font_h3.borrow().clone()
    }

    /// Sets the font used to render H3 heading elements
    pub fn set_font_h3(&self, font: FontDescription) {
        *self.imp().font_h3.borrow_mut() = font;
    }

    /// Renders plain text
    pub fn render_text(&self, data: &str) {
        self.clear();
        let buf = self.buffer();
        let prebox = gtk::builders::BoxBuilder::new()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .halign(gtk::Align::Fill)
            .margin_bottom(8)
            .margin_top(8)
            .margin_start(8)
            .margin_end(8)
            .css_classes(vec!["preformatted".to_string()])
            .build();
        let mut iter = buf.end_iter();
        let anchor = buf.create_child_anchor(&mut iter);
        self.add_child_at_anchor(&prebox, &anchor);
        let text = glib::markup_escape_text(data);
        let font = self.font_pre();
        let label = gtk::builders::LabelBuilder::new()
            .use_markup(true)
            .css_classes(vec!["preformatted".to_string()])
            .label(&format!(
                "<span font=\"{}\">{}</span>",
                font.to_str(),
                &text,
            ))
            .build();
        prebox.append(&label);
    }

    /// Renders a Vec<u8> into an image
    pub fn render_image_from_bytes(&self, bytes: &Vec<u8>) {
        let bytes = gtk::glib::Bytes::from(bytes);
        let stream = MemoryInputStream::from_bytes(&bytes);
        if let Ok(pixbuf) = Pixbuf::from_stream(&stream, Option::<&Cancellable>::None) {
            let img = self.render_pixbuf(&pixbuf);
            img.set_pixel_size(self.height() - 50);
        }
    }

    /// Renders a [`gtk::gdk_pixbuf::Pixbuf`]
    pub fn render_pixbuf(&self, pixbuf: &gtk::gdk_pixbuf::Pixbuf) -> gtk::Image {
        self.clear();
        let buf = self.buffer();
        let mut iter = buf.end_iter();
        let anchor = buf.create_child_anchor(&mut iter);
        let image = gtk::Image::from_pixbuf(Some(pixbuf));
        image.set_hexpand(true);
        image.set_halign(gtk::Align::Fill);
        image.set_css_classes(&["image"]);
        self.add_child_at_anchor(&image, &anchor);
        image
    }

    /// Renders the given `&str` as a gemtext document
    pub fn render_gmi(&self, data: &str) {
        self.clear();
        let buf = self.buffer();
        let mut iter;
        let mut preflag = false;
        let mut prebox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let nodes = gemtext::parse_gemtext(data);
        for node in nodes {
            match node {
                GemtextNode::Text(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">{}</span>\n",
                            font.to_str(),
                            self.wrap_text(&text),
                        ),
                    );
                }
                GemtextNode::Heading(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_h1();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">{}</span>\n",
                            font.to_str(),
                            self.wrap_text(&text),
                        ),
                    );
                }
                GemtextNode::SubHeading(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_h2();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">{}</span>\n",
                            font.to_str(),
                            self.wrap_text(&text),
                        ),
                    );
                }
                GemtextNode::SubSubHeading(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_h3();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">{}</span>\n",
                            font.to_str(),
                            self.wrap_text(&text),
                        ),
                    );
                }
                GemtextNode::ListItem(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">  • {}</span>\n",
                            font.to_str(),
                            self.wrap_text(&text),
                        ),
                    );
                }
                GemtextNode::Link(link, text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_paragraph();
                    iter = buf.end_iter();
                    let link = link.replace('&', "&amp;");
                    let anchor = buf.create_child_anchor(&mut iter);
                    let label = gtk::builders::LabelBuilder::new()
                        .use_markup(true)
                        .tooltip_text(&if link.len() < 80 {
                            link.clone()
                        } else {
                            format!("{}...", &link[..80])
                        })
                        .label(&format!(
                            "<span font=\"{}\"><a href=\"{}\">{}</a></span>\n",
                            font.to_str(),
                            &link,
                            match text {
                                Some(t) => self.wrap_text(&t),
                                None => self.wrap_text(&link),
                            },
                        ))
                        .build();
                    label.set_cursor_from_name(Some("pointer"));
                    let open_menu = Menu::new();
                    let in_tab = MenuItem::new(Some("Open link in new tab"), None);
                    let in_window = MenuItem::new(Some("Open link in new window"), None);
                    open_menu.append_item(&in_tab);
                    open_menu.append_item(&in_window);
                    label.set_extra_menu(Some(&open_menu));
                    self.add_child_at_anchor(&label, &anchor);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                    let viewer = self.clone();
                    label.connect_activate_link(move |_, link| {
                        if let Err(e) = viewer.visit(link) {
                            eprintln!("Error: {}", e);
                            let estr = format!("{:?}", e);
                            viewer.emit_by_name::<()>("page-load-failed", &[&estr]);
                        };
                        gtk::Inhibit(true)
                    });
                }
                GemtextNode::Blockquote(text) => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let font = self.font_quote();
                    iter = buf.end_iter();
                    let anchor = buf.create_child_anchor(&mut iter);
                    let quotebox = gtk::builders::BoxBuilder::new()
                        .orientation(gtk::Orientation::Vertical)
                        .hexpand(true)
                        .halign(gtk::Align::Fill)
                        .margin_bottom(8)
                        .margin_top(8)
                        .margin_start(8)
                        .margin_end(8)
                        .css_classes(vec!["blockquote".to_string()])
                        .build();
                    let label = gtk::builders::LabelBuilder::new()
                        .use_markup(true)
                        .css_classes(vec!["blockquote".to_string()])
                        .label(&format!(
                            "<span font=\"{}\">{}</span>",
                            font.to_str(),
                            self.wrap_text(&text),
                        ))
                        .build();
                    quotebox.append(&label);
                    self.add_child_at_anchor(&quotebox, &anchor);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                }
                GemtextNode::Preformatted(text, _) => {
                    if !preflag {
                        prebox = gtk::builders::BoxBuilder::new()
                            .orientation(gtk::Orientation::Vertical)
                            .hexpand(true)
                            .halign(gtk::Align::Fill)
                            .margin_bottom(8)
                            .margin_top(8)
                            .margin_start(8)
                            .margin_end(8)
                            .css_classes(vec!["preformatted".to_string()])
                            .build();
                        iter = buf.end_iter();
                        let anchor = buf.create_child_anchor(&mut iter);
                        self.add_child_at_anchor(&prebox, &anchor);
                    }
                    let font = self.font_pre();
                    let label = gtk::builders::LabelBuilder::new()
                        .use_markup(true)
                        .css_classes(vec!["preformatted".to_string()])
                        .label(&format!(
                            "<span font=\"{}\">{}</span>",
                            font.to_str(),
                            &text,
                        ))
                        .build();
                    prebox.append(&label);
                }
                GemtextNode::EmptyLine => {
                    if preflag {
                        iter = buf.end_iter();
                        buf.insert(&mut iter, "\n");
                        preflag = false;
                    }
                    let mut iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                }
            }
        }
    }

    fn render_gopher(&self, content: &gopher::Content) {
        self.clear();
        let buf = self.buffer();
        let mut iter;
        for line in content.parse() {
            iter = buf.end_iter();
            match line {
                gopher::parser::LineType::Text(text) => {
                    buf.insert(&mut iter, &text);
                    buf.insert(&mut iter, "\n");
                },
                gopher::parser::LineType::Link(link) => {
                    buf.insert(&mut iter, &link.to_markup(&self.font_pre()));
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

    fn absolute_url(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        match url::Url::parse(url) {
            Ok(u) => match u.scheme() {
                "gemini" | "mercury" | "data" => Ok(url.to_string()),
                s => {
                    self.emit_by_name::<()>("request-unsupported-scheme", &[&url.to_string()]);
                    Err(format!("unsupported-scheme: {}", s).into())
                }
            },
            Err(e) => match e {
                url::ParseError::RelativeUrlWithoutBase => {
                    let origin = url::Url::parse(&self.uri())?;
                    let new = origin.join(url)?;
                    Ok(new.to_string())
                }
                _ => Err(e.into()),
            },
        }
    }

    /// Parse the given uri and then visits the page
    pub fn visit(&self, addr: &str) -> Result<(), Box<dyn Error>> {
        let uri = self.load(addr)?;
        if let Some(uri) = uri {
            self.append_history(&uri);
            self.emit_by_name::<()>("page-loaded", &[&uri]);
        }
        Ok(())
    }

    fn load(&self, addr: &str) -> Result<Option<String>, Box<dyn Error>> {
        self.emit_by_name::<()>("page-load-started", &[&addr]);
        let abs = match self.absolute_url(addr) {
            Ok(s) => s,
            Err(e) => {
                if format!("{:?}", e).contains("unsupported-scheme") {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
        };
        if abs.starts_with("data:") {
            if let Some(url) = self.load_data(&abs)? {
                return Ok(Some(url));
            } else {
                return Ok(None);
            }
        }
        let mut uri = match Url::try_from(abs.as_str()) {
            Ok(u) => u,
            Err(e) => {
                let estr = format!("{:?}", e);
                self.emit_by_name::<()>("page-load-failed", &[&estr]);
                return Err(e.into());
            }
        };
        loop {
            let response = match request::make_request(&uri) {
                Ok(r) => r,
                Err(e) => {
                    let estr = format!("{:?}", e);
                    self.emit_by_name::<()>("page-load-failed", &[&estr]);
                    return Err(e.into());
                }
            };
            match response.status {
                protocol::StatusCode::Redirect(c) => {
                    println!("Redirect code {} with meta {}", c, response.meta);
                    uri = match Url::try_from(response.meta.as_str()) {
                        Ok(r) => r,
                        Err(e) => {
                            let estr = format!("{:?}", e);
                            self.emit_by_name::<()>("page-load-failed", &[&estr]);
                            return Err(e.into());
                        }
                    };
                }
                protocol::StatusCode::Success(_) => {
                    self.set_buffer_mime(&response.meta);
                    self.set_buffer_content(&response.data);
                    if response.meta.starts_with("text/gemini") {
                        self.render_gmi(&String::from_utf8_lossy(&response.data));
                    } else if response.meta.starts_with("text/plain") {
                        self.render_text(&String::from_utf8_lossy(&response.data));
                    } else if response.meta.starts_with("image") {
                        self.render_image_from_bytes(&response.data);
                    } else {
                        self.emit_by_name::<()>("request-download", &[&response.meta]);
                    }
                    let uri_str = uri.to_string();
                    return Ok(Some(uri_str));
                }
                s => {
                    let estr = format!("{:?}", s);
                    self.emit_by_name::<()>("page-load-failed", &[&estr]);
                    return Err(String::from("unknown-response-code").into());
                }
            }
        }
    }

    fn load_data(&self, url: &str) -> Result<Option<String>, Box<dyn Error>> {
        let data = DataUrl::try_from(url)?;
        match data.mime() {
            MimeType::TextPlain => {
                if let Data::Text(payload) = data.decode()? {
                    self.render_text(&payload);
                    Ok(Some(url.to_string()))
                } else {
                    unreachable!();
                }
            }
            MimeType::TextGemini => {
                if let Data::Text(payload) = data.decode()? {
                    self.render_gmi(&payload);
                    Ok(Some(url.to_string()))
                } else {
                    unreachable!();
                }
            }
            MimeType::ImagePng | MimeType::ImageJpeg | MimeType::ImageSvg => {
                if let Data::Bytes(payload) = data.decode()? {
                    self.render_image_from_bytes(&payload);
                    Ok(Some(url.to_string()))
                } else {
                    unreachable!();
                }
            }
            _ => Err(String::from("Unrecognized data type").into()),
        }
    }

    /// Reloads the current page
    ///
    /// ## Errors
    /// Propagates ay page load errors
    pub fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        let uri = self.load(&self.uri())?;
        self.emit_by_name::<()>("page-loaded", &[&uri]);
        Ok(())
    }

    /// Connects to the "page-load-started" signal, emitted when the browser
    /// begins loading a uri
    pub fn connect_page_load_started<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("page-load-started", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = obj.uri();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "page-load-redirect" signal, emitted during a page load
    /// whenever the browser encounters a redirect
    pub fn connect_page_load_redirect<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("page-load-redirect", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = obj.uri();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "page-load-failed" signal, emitted whenever a page has
    /// failed to load
    pub fn connect_page_load_failed<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("page-load-failed", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = obj.uri();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "page-loaded" signal, emitted when the browser has
    /// successfully loaded a page
    pub fn connect_page_loaded<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("page-loaded", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = obj.uri();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "request-unsupported-scheme" signal, emitted when the
    /// browser has had a request to load a page with an unsupported scheme
    pub fn connect_request_unsupported_scheme<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-unsupported-scheme", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = values[1].get::<String>().unwrap();
            f(&obj, uri);
            None
        })
    }

    fn wrap_text(&self, text: &str) -> String {
        let width: usize = match self.root() {
            Some(win) => std::cmp::min((win.width() / 10).try_into().unwrap(), 175),
            None => 175,
        };
        fill(glib::markup_escape_text(text).as_str(), width)
    }
}
