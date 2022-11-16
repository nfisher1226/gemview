#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![doc = include_str!("../README.md")]

use {
    glib::{Continue, MainContext, Object, PRIORITY_DEFAULT},
    gtk::{
        gdk_pixbuf::Pixbuf,
        gio::{Cancellable, MemoryInputStream, Menu, MenuItem, SimpleAction, SimpleActionGroup},
        glib,
        pango::FontDescription,
        prelude::*,
        subclass::prelude::*,
    },
    std::{borrow::Cow, path::PathBuf, thread},
    textwrap::fill,
    url::Url,
};

mod imp;
pub mod scheme;
mod upload;
use {
    data::{Data, DataUrl, MimeType},
    gemini::parser::GemtextNode,
    gopher::GopherMap,
    scheme::{data, finger, gemini, gopher, spartan, Content, Response, ToLabel},
    upload::UploadWidget,
};

enum TextSize {
    Paragraph,
    H1,
    H2,
    H3,
}

glib::wrapper! {
/// The gemini browser widget is a subclass of the `TextView` widget which
/// has been customized for browsing [geminispace](https://gemini.circumlunar.space).
pub struct GemView(ObjectSubclass<imp::GemView>)
    @extends gtk::TextView, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl Default for GemView {
    fn default() -> Self {
        Self::new()
    }
}

impl GemView {
    #[allow(clippy::must_use_candidate)]
    pub fn new() -> Self {
        Object::new(&[
            ("margin-start", &45.to_value()),
            ("margin-end", &45.to_value()),
            ("margin-top", &25.to_value()),
            ("margin-bottom", &25.to_value()),
            ("wrap-mode", &gtk::WrapMode::Word),
        ])
    }

    #[allow(clippy::must_use_candidate)]
    pub fn with_label(label: &str) -> Self {
        Object::new(&[
            ("label", &label),
            ("margin-start", &45.to_value()),
            ("margin-end", &45.to_value()),
            ("margin-top", &25.to_value()),
            ("margin-bottom", &25.to_value()),
            ("wrap-mode", &gtk::WrapMode::Word),
        ])
    }

    fn add_actions(&self) {
        let request_new_tab = SimpleAction::new("request-new-tab", Some(glib::VariantTy::STRING));
        let request_new_window =
            SimpleAction::new("request-new-window", Some(glib::VariantTy::STRING));
        let group = SimpleActionGroup::new();
        group.add_action(&request_new_tab);
        group.add_action(&request_new_window);
        let viewer = self.clone();
        request_new_tab.connect_activate(move |_, url| {
            if let Some(url) = url {
                if let Some(url) = url.get::<String>() {
                    if let Ok(url) = urlencoding::decode(&url) {
                        if let Ok(url) = viewer.absolute_url(&url) {
                            viewer.emit_by_name::<()>("request-new-tab", &[&url.to_string()]);
                        }
                    }
                }
            }
        });
        let viewer = self.clone();
        request_new_window.connect_activate(move |_, url| {
            if let Some(url) = url {
                if let Some(url) = url.get::<String>() {
                    if let Ok(url) = urlencoding::decode(&url) {
                        if let Ok(url) = viewer.absolute_url(&url) {
                            viewer.emit_by_name::<()>("request-new-window", &[&url.to_string()]);
                        }
                    }
                }
            }
        });
        self.insert_action_group("viewer", Some(&group));
    }

    #[must_use]
    /// Returns the current uri
    pub fn uri(&self) -> String {
        self.imp().history.borrow().uri.clone()
    }

    /// Sets the current uri
    pub fn set_uri(&self, uri: &str) {
        self.imp().history.borrow_mut().uri = String::from(uri);
    }

    /// Visits the previous page, if there is one
    fn previous(&self) -> Option<String> {
        self.imp().history.borrow_mut().previous()
    }

    #[must_use]
    /// Returns `true` if there are any items in the `back` history list
    pub fn has_previous(&self) -> bool {
        self.imp().history.borrow().has_previous()
    }

    /// If there are any items in the `back` history list, retrieves the most
    /// recent one and visits that uri
    pub fn go_previous(&self) {
        if let Some(uri) = self.previous() {
            self.load(&uri);
        }
    }

    fn next(&self) -> Option<String> {
        let imp = self.imp();
        imp.history.borrow_mut().next()
    }

    #[must_use]
    /// Returns `true` if there are any items in the `forward` history list
    pub fn has_next(&self) -> bool {
        self.imp().history.borrow().has_next()
    }

    /// If there are any items in the `forward` history list, retrieves the most
    /// recent item and visits that uri
    pub fn go_next(&self) {
        if let Some(uri) = self.next() {
            self.load(&uri);
        }
    }

    /// Manually appends an item into the browser's history. Normally this function
    /// will not need to be called directly.
    pub fn append_history(&self, uri: &str) {
        let current = self.uri();
        if current != uri {
            self.imp().history.borrow_mut().append(uri.to_string());
        }
    }

    #[must_use]
    /// Get the `MimeType` of the current file
    pub fn buffer_mime(&self) -> String {
        self.imp().buffer.borrow().mime.clone()
    }

    /// Set the `MimeType` of the current file. Normally this function will not
    /// need to be called directly.
    pub fn set_buffer_mime(&self, mime: &str) {
        self.imp().buffer.borrow_mut().mime = mime.to_string();
    }

    #[must_use]
    /// Get the contents of the buffer. Can be used to save the current page
    /// source.
    pub fn buffer_content(&self) -> Vec<u8> {
        self.imp().buffer.borrow().content.clone()
    }

    /// Set the contents of the buffer. Normally this function will not need to
    /// be called directly
    pub fn set_buffer_content(&self, content: &[u8]) {
        self.imp().buffer.borrow_mut().content = content.to_vec();
    }

    #[must_use]
    /// Returns the font used to render "normal" elements
    pub fn font_paragraph(&self) -> FontDescription {
        self.imp().font_paragraph.borrow().clone()
    }

    /// Sets the font used to render "normal" elements
    pub fn set_font_paragraph(&self, font: FontDescription) {
        let tag = self.imp().paragraph_tag.borrow_mut();
        tag.set_font_desc(Some(&font));
        *self.imp().font_paragraph.borrow_mut() = font;
    }

    #[must_use]
    /// Returns the font used to render "preformatted" elements
    pub fn font_pre(&self) -> FontDescription {
        self.imp().font_pre.borrow().clone()
    }

    /// Sets the font used to render "preformatted" elements
    pub fn set_font_pre(&self, font: FontDescription) {
        *self.imp().font_pre.borrow_mut() = font;
    }

    #[must_use]
    /// Returns the font used to render "blockte" elements
    pub fn font_quote(&self) -> FontDescription {
        self.imp().font_quote.borrow().clone()
    }

    /// Sets the font used to render "blockquote" elements
    pub fn set_font_quote(&self, font: FontDescription) {
        *self.imp().font_quote.borrow_mut() = font;
    }

    #[must_use]
    /// Returns the font used to render H1 heading elements
    pub fn font_h1(&self) -> FontDescription {
        self.imp().font_h1.borrow().clone()
    }

    /// Sets the font used to render H1 heading elements
    pub fn set_font_h1(&self, font: FontDescription) {
        let tag = self.imp().h1_tag.borrow_mut();
        tag.set_font_desc(Some(&font));
        *self.imp().font_h1.borrow_mut() = font;
    }

    #[must_use]
    /// Returns the font used to render H2 heading elements
    pub fn font_h2(&self) -> FontDescription {
        self.imp().font_h2.borrow().clone()
    }

    /// Sets the font used to render H2 heading elements
    pub fn set_font_h2(&self, font: FontDescription) {
        let tag = self.imp().h2_tag.borrow_mut();
        tag.set_font_desc(Some(&font));
        *self.imp().font_h2.borrow_mut() = font;
    }

    #[must_use]
    /// Returns the font used to render H3 heading elements
    pub fn font_h3(&self) -> FontDescription {
        self.imp().font_h3.borrow().clone()
    }

    /// Sets the font used to render H3 heading elements
    pub fn set_font_h3(&self, font: FontDescription) {
        let tag = self.imp().h3_tag.borrow_mut();
        tag.set_font_desc(Some(&font));
        *self.imp().font_h3.borrow_mut() = font;
    }

    fn get_iter(&self) -> (gtk::TextBuffer, gtk::TextIter) {
        let buf = self.buffer();
        let iter = buf.end_iter();
        (buf, iter)
    }

    /// Renders plain text
    pub fn render_text(&self, data: &str) {
        self.clear();
        let (buf, mut iter) = self.get_iter();
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
    fn render_pixbuf(&self, pixbuf: &gtk::gdk_pixbuf::Pixbuf) -> gtk::Image {
        self.clear();
        let (buf, mut iter) = self.get_iter();
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
        let nodes = gemini::parser::Parser::default().parse(data);
        for node in nodes {
            match node {
                GemtextNode::Text(text) => {
                    self.insert_text_block(text, TextSize::Paragraph);
                }
                GemtextNode::H1(text) => {
                    self.insert_text_block(text, TextSize::H1);
                }
                GemtextNode::H2(text) => {
                    self.insert_text_block(text, TextSize::H2);
                }
                GemtextNode::H3(text) => {
                    self.insert_text_block(text, TextSize::H3);
                }
                GemtextNode::ListItem(text) => {
                    self.insert_list_item(text);
                }
                GemtextNode::Link(link) => {
                    self.insert_link(link.url, link.display);
                }
                GemtextNode::Prompt(link) => {
                    self.insert_prompt_link(link.url, link.display);
                }
                GemtextNode::Blockquote(text) => {
                    let font = self.font_quote();
                    let (buf, mut iter) = self.get_iter();
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
                        .selectable(true)
                        .use_markup(true)
                        .css_classes(vec!["blockquote".to_string()])
                        .label(&format!(
                            "<span font=\"{}\">{}</span>",
                            font.to_str(),
                            self.wrap_text(&text, self.font_paragraph().size()),
                        ))
                        .build();
                    quotebox.append(&label);
                    self.add_child_at_anchor(&quotebox, &anchor);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                }
                GemtextNode::Preformatted(text, _) => {
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
                    let (buf, mut iter) = self.get_iter();
                    let anchor = buf.create_child_anchor(&mut iter);
                    self.add_child_at_anchor(&prebox, &anchor);
                    let font = self.font_pre();
                    let label = gtk::builders::LabelBuilder::new()
                        .selectable(true)
                        .use_markup(true)
                        .css_classes(vec!["preformatted".to_string()])
                        .label(&format!(
                            "<span font=\"{}\">{}</span>",
                            font.to_str(),
                            glib::markup_escape_text(&text)
                        ))
                        .build();
                    prebox.append(&label);
                    iter = buf.end_iter();
                    buf.insert(&mut iter, "\n");
                }
            }
        }
    }

    fn insert_text_block(&self, text: &str, size: TextSize) {
        let (buf, mut iter) = self.get_iter();
        let tag = match size {
            TextSize::Paragraph => self.imp().paragraph_tag.borrow(),
            TextSize::H1 => self.imp().h1_tag.borrow(),
            TextSize::H2 => self.imp().h2_tag.borrow(),
            TextSize::H3 => self.imp().h3_tag.borrow(),
        };
        buf.insert_with_tags(&mut iter, text, &[&tag]);
        /*buf.insert_markup(
            &mut iter,
            &format!(
                "<span font=\"{}\">{}</span>",
                font.to_str(),
                self.wrap_text(text, font.size()),
            ),
        );*/
        iter = buf.end_iter();
        buf.insert(&mut iter, "\n");
    }

    fn insert_list_item(&self, text: &str) {
        let (buf, mut iter) = self.get_iter();
        let font = self.font_paragraph();
        buf.insert_markup(
            &mut iter,
            &format!(
                "<span font=\"{}\">  • {}</span>",
                font.to_str(),
                self.wrap_text(text, self.font_paragraph().size()),
            ),
        );
        iter = buf.end_iter();
        buf.insert(&mut iter, "\n");
    }

    fn insert_link(&self, link: &str, text: Option<String>) {
        let u = self.uri();
        let (old, _) = u.split_once(':').unwrap_or(("gemini", ""));
        let (scheme, _) = link.split_once(':').unwrap_or((old, ""));
        let start = match scheme {
            "gemini" => "<span color=\"#0000ff\"> 🛰️  </span>",
            "spartan" => "<span color=\"#0000ff\"> 🗡️ </span>",
            "gopher" => "<span color=\"#00ff00\"> 🕳️  </span>",
            "finger" => "<span color=\"#00ffff\"> 👉 </span>",
            "data" => "<span color=\"#ff00ff\"> 📊 </span>",
            "http" | "https" => "<span color=\"#ff0000\"> 🌐  </span>",
            "mailto" => "<span color=\"#ffff00\"> ✉️ </span>",
            "file" => "<span color=\"#0000ff\"> 🗄️ </span>",
            _ => "<span color=\"#ffff00\"> 🌐  </span>",
        };
        let label = self.insert_gmi_link_markup_label(start, link, text);
        label.set_extra_menu(Some(&Self::context_menu(link)));
        let viewer = self.clone();
        label.connect_activate_link(move |_, link| {
            viewer.visit(link);
            gtk::Inhibit(true)
        });
    }

    fn insert_prompt_link(&self, link: &str, text: Option<String>) {
        match self.uri().split_once(':') {
            Some((s, _)) if s == "spartan" => {
                let start = "<span color=\"#0000ff\"> 📤  </span>";
                let label = self.insert_gmi_link_markup_label(start, link, text);
                let viewer = self.clone();
                label.connect_activate_link(move |_, link| {
                    let url = if let Some(("spartan", _)) = link.split_once(':') {
                        link.to_string()
                    } else {
                        let u = viewer.uri();
                        let u = Url::parse(&u).unwrap();
                        u.join(link).unwrap().to_string()
                    };
                    viewer.set_uri(&url);
                    viewer.emit_by_name::<()>("request-upload", &[&url]);
                    gtk::Inhibit(true)
                });
            }
            _ => {
                let text = if let Some(t) = text {
                    Cow::from(format!("=: {link} {t}"))
                } else {
                    Cow::from(link)
                };
                self.insert_text_block(&text, TextSize::Paragraph);
            }
        }
    }

    fn insert_gmi_link_markup_label(
        &self,
        start: &str,
        link: &str,
        text: Option<String>,
    ) -> gtk::Label {
        let (buf, mut iter) = self.get_iter();
        let link = link.replace('&', "&amp;");
        let anchor = buf.create_child_anchor(&mut iter);
        let label = gtk::builders::LabelBuilder::new()
            .use_markup(true)
            .label(&format!(
                "{}<span font=\"{}\"><a href=\"{}\">{}</a></span>",
                start,
                self.font_paragraph().to_str(),
                &link,
                match text {
                    Some(t) => self.wrap_text(&t, self.font_paragraph().size()),
                    None => self.wrap_text(&link, self.font_paragraph().size()),
                },
            ))
            .tooltip_text(&if link.len() < 80 {
                link
            } else {
                format!("{}...", &link[..80])
            })
            .build();
        label.set_cursor_from_name(Some("pointer"));
        self.add_child_at_anchor(&label, &anchor);
        iter = buf.end_iter();
        buf.insert(&mut iter, "\n");
        label
    }

    /// Renders a `GopherMap`
    fn render_gopher(&self, content: &scheme::Content) {
        self.clear();
        let buf = self.buffer();
        let mut iter;
        for line in content.parse() {
            iter = buf.end_iter();
            match line {
                gopher::parser::LineType::Text(text) => {
                    buf.insert_markup(
                        &mut iter,
                        &format!(
                            "<span font=\"{}\">{}</span>\n",
                            &self.font_pre(),
                            glib::markup_escape_text(&text)
                        ),
                    );
                }
                gopher::parser::LineType::Link(link) => {
                    let label = link.to_label(&self.font_pre());
                    self.insert_gopher_link(&label);
                    label.set_extra_menu(Some(&Self::context_menu(&link.to_string())));
                    let viewer = self.clone();
                    label.connect_activate_link(move |_, link| {
                        viewer.visit(link);
                        gtk::Inhibit(true)
                    });
                }
                gopher::parser::LineType::Query(link) => {
                    let label = link.to_label(&self.font_pre());
                    self.insert_gopher_link(&label);
                    let viewer = self.clone();
                    label.connect_activate_link(move |_, link| {
                        viewer.emit_by_name::<()>(
                            "request-input",
                            &[&String::from("Enter query"), &link],
                        );
                        gtk::Inhibit(true)
                    });
                }
                gopher::parser::LineType::Http(link) => {
                    let label = link.to_label(&self.font_pre());
                    self.insert_gopher_link(&label);
                    label.set_extra_menu(Some(&Self::context_menu(&link.url)));
                    let viewer = self.clone();
                    label.connect_activate_link(move |_, link| {
                        viewer.visit(link);
                        gtk::Inhibit(true)
                    });
                }
            }
        }
    }

    fn insert_gopher_link(&self, label: &gtk::Label) {
        let (buf, mut iter) = self.get_iter();
        let anchor = buf.create_child_anchor(&mut iter);
        self.add_child_at_anchor(label, &anchor);
        iter = buf.end_iter();
        buf.insert(&mut iter, "\n");
    }

    fn context_menu(link: &str) -> Menu {
        let menu = Menu::new();
        let url = urlencoding::encode(link);
        let action_name = format!("viewer.request-new-tab('{}')", &url);
        let in_tab = MenuItem::new(Some("Open in new tab"), Some(&action_name));
        let action_name = format!("viewer.request-new-window('{}')", &url);
        let in_window = MenuItem::new(Some("Open in new window"), Some(&action_name));
        menu.append_item(&in_tab);
        menu.append_item(&in_window);
        menu
    }

    /// Clears the text buffer
    pub fn clear(&self) {
        let buf = self.buffer();
        let (mut start, mut end) = buf.bounds();
        buf.delete(&mut start, &mut end);
    }

    fn absolute_url(&self, url: &str) -> Result<Url, Box<dyn std::error::Error>> {
        match Url::parse(url) {
            Ok(u) => match u.scheme() {
                "gemini" | "mercury" | "data" | "gopher" | "finger" | "file" | "spartan" => Ok(u),
                s => {
                    self.emit_by_name::<()>("request-unsupported-scheme", &[&url.to_string()]);
                    Err(format!("unsupported-scheme: {s}").into())
                }
            },
            Err(e) => match e {
                url::ParseError::RelativeUrlWithoutBase => {
                    let origin = url::Url::parse(&self.uri())?;
                    let new = origin.join(url)?;
                    Ok(new)
                }
                _ => Err(e.into()),
            },
        }
    }

    /// Parse the given uri and then visits the page
    pub fn visit(&self, addr: &str) {
        self.load(addr);
    }

    fn load(&self, addr: &str) {
        self.emit_by_name::<()>("page-load-started", &[&addr]);
        let url = match self.absolute_url(addr) {
            Ok(s) => s,
            Err(e) => {
                let estr = format!("{e:?}");
                self.emit_by_name::<()>("page-load-failed", &[&estr]);
                return;
            }
        };
        match url.scheme() {
            "data" => self.load_data(&url),
            "gemini" => self.load_gemini(url),
            "gopher" => self.load_gopher(url),
            "file" => self.load_file(&url),
            "finger" => self.load_finger(url),
            "spartan" => self.load_spartan(url),
            _ => {}
        }
    }

    fn load_data(&self, url: &Url) {
        let data = match DataUrl::try_from(url.to_string().as_str()) {
            Ok(d) => d,
            Err(e) => {
                let estr = format!("{e:?}");
                self.emit_by_name::<()>("page-load-failed", &[&estr]);
                return;
            }
        };
        match data.mime() {
            MimeType::TextPlain => match data.decode() {
                Ok(Data::Text(payload)) => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime("text/plain");
                    self.set_buffer_content(payload.as_bytes());
                    self.render_text(&payload);
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                _ => unreachable!(),
            },
            MimeType::TextGemini => match data.decode() {
                Ok(Data::Text(payload)) => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime("text/gemini");
                    self.set_buffer_content(payload.as_bytes());
                    self.render_gmi(&payload);
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                _ => unreachable!(),
            },
            MimeType::ImagePng
            | MimeType::ImageJpeg
            | MimeType::ImageSvg
            | MimeType::ImageOther => match data.decode() {
                Ok(Data::Bytes(payload)) => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime(match data.mime() {
                        MimeType::ImageJpeg => "image/jpeg",
                        MimeType::ImageSvg => "image/svg+xml",
                        MimeType::ImagePng => "image/png",
                        MimeType::ImageOther => "image/other",
                        _ => unreachable!(),
                    });
                    self.set_buffer_content(&payload);
                    self.render_image_from_bytes(&payload);
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                _ => unreachable!(),
            },
            MimeType::Unknown => self
                .emit_by_name::<()>("page-load-failed", &[&"unrecognized data type".to_string()]),
        }
    }

    fn load_file(&self, url: &Url) {
        let path = PathBuf::from(url.path());
        if let Some(mime) = tree_magic_mini::from_filepath(&path) {
            if !mime.starts_with("text/")
                && !mime.starts_with("image/")
                && mime != "inode/directory"
            {
                if let Err(e) = mime_open::open(url.as_ref()) {
                    eprintln!("{e}");
                }
                self.emit_by_name::<()>("page-loaded", &[&url.to_string()]);
                return;
            }
        }
        if let Ok(content) = Content::try_from(url.clone()) {
            match content.mime {
                s if s.starts_with("text/gemini") => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime(&s);
                    self.set_buffer_content(&content.bytes);
                    self.render_gmi(&String::from_utf8_lossy(&content.bytes));
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                s if s.starts_with("text/") => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime(&s);
                    self.set_buffer_content(&content.bytes);
                    self.render_text(&String::from_utf8_lossy(&content.bytes));
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                s if s.starts_with("image/") => {
                    let url = url.to_string();
                    self.append_history(&url);
                    self.set_buffer_mime(&s);
                    self.set_buffer_content(&content.bytes);
                    self.render_image_from_bytes(&content.bytes);
                    self.emit_by_name::<()>("page-loaded", &[&url]);
                }
                _ => {}
            }
        }
    }

    fn load_gopher(&self, url: Url) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let req = url.clone();
        thread::spawn(move || match gopher::request(&req) {
            Ok(content) => {
                sender
                    .send(Response::Success(content))
                    .expect("Cannot send data");
            }
            Err(e) => {
                sender
                    .send(Response::Error(format!("{e:?}")))
                    .expect("Cannot send data");
            }
        });
        let viewer = self.clone();
        receiver.attach(None, move |response| {
            match response {
                Response::Success(content) => {
                    viewer.set_buffer_mime(&content.mime);
                    viewer.set_buffer_content(&content.bytes);
                    if content.mime.starts_with("text") {
                        let url = url.to_string();
                        viewer.append_history(&url);
                        if content.is_map() {
                            viewer.render_gopher(&content);
                        } else {
                            viewer.render_text(&String::from_utf8_lossy(&content.bytes));
                        }
                        viewer.emit_by_name::<()>("page-loaded", &[&url]);
                    } else if content.mime.starts_with("image") {
                        let url = url.to_string();
                        viewer.append_history(&url);
                        viewer.render_image_from_bytes(&content.bytes);
                        viewer.emit_by_name::<()>("page-loaded", &[&url]);
                    } else {
                        let filename = if let Some(segments) = url.path_segments() {
                            segments.last().unwrap_or("download")
                        } else {
                            "download"
                        }
                        .to_string();

                        viewer.emit_by_name::<()>("request-download", &[&content.mime, &filename]);
                    }
                }
                Response::Error(err) => {
                    viewer.emit_by_name::<()>("page-load-failed", &[&err]);
                }
                _ => unreachable!(),
            }
            Continue(false)
        });
    }

    fn load_finger(&self, url: Url) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let req = url.clone();
        thread::spawn(move || match finger::request(&req) {
            Ok(content) => {
                sender
                    .send(Response::Success(content))
                    .expect("Cannot send data");
            }
            Err(e) => {
                sender
                    .send(Response::Error(format!("{e:?}")))
                    .expect("Cannot send data");
            }
        });
        let viewer = self.clone();
        receiver.attach(None, move |response| {
            match response {
                Response::Success(content) => {
                    let url = url.to_string();
                    viewer.append_history(&url);
                    viewer.set_buffer_mime(&content.mime);
                    viewer.set_buffer_content(&content.bytes);
                    viewer.render_text(&String::from_utf8_lossy(&content.bytes));
                    viewer.emit_by_name::<()>("page-loaded", &[&url]);
                }
                Response::Error(err) => {
                    viewer.emit_by_name::<()>("page-load-failed", &[&err]);
                }
                _ => unreachable!(),
            }
            Continue(false)
        });
    }

    fn load_spartan(&self, url: Url) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let u = url.clone();
        thread::spawn(move || {
            let mut url = u;
            loop {
                let response = match spartan::request(&url) {
                    Ok(r) => r,
                    Err(e) => {
                        let estr = format!("{e:?}");
                        sender
                            .send(scheme::Response::Error(estr))
                            .expect("Cannot send data");
                        break;
                    }
                };
                let msg = response.into_message(&mut url);
                if let Response::Redirect(_) = msg {
                    continue;
                };
                sender.send(msg).expect("Cannot send message");
                break;
            }
        });
        let viewer = self.clone();
        receiver.attach(None, move |response| {
            match response {
                scheme::Response::Success(content) => {
                    viewer.process_gemini_response_success(&content, &url);
                }
                scheme::Response::Error(estr) => {
                    viewer.emit_by_name::<()>("page-load-failed", &[&estr]);
                }
                _ => unreachable!(),
            }
            Continue(false)
        });
    }

    pub fn post_spartan(&self, url: Url, data: Vec<u8>) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let u = url.clone();
        thread::spawn(move || {
            let mut url = u;
            loop {
                let response = match spartan::post(&url, &data) {
                    Ok(r) => r,
                    Err(e) => {
                        let estr = format!("{e:?}");
                        sender
                            .send(scheme::Response::Error(estr))
                            .expect("Cannot send data");
                        break;
                    }
                };
                let msg = response.into_message(&mut url);
                if let Response::Redirect(_) = msg {
                    continue;
                };
                sender.send(msg).expect("Cannot send message");
                break;
            }
        });
        let viewer = self.clone();
        receiver.attach(None, move |response| {
            match response {
                scheme::Response::Success(content) => {
                    viewer.process_gemini_response_success(&content, &url);
                }
                scheme::Response::Redirect(_s) => {}
                scheme::Response::Error(estr) => {
                    viewer.emit_by_name::<()>("page-load-failed", &[&estr]);
                }
                scheme::Response::RequestInput(_) => unreachable!(),
            }
            Continue(false)
        });
    }

    fn load_gemini(&self, url: Url) {
        let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
        let u = url.clone();
        thread::spawn(move || {
            let mut url = u;
            loop {
                let response = match gemini::request::request(&url) {
                    Ok(r) => r,
                    Err(e) => {
                        let estr = format!("{e:?}");
                        sender
                            .send(scheme::Response::Error(estr))
                            .expect("Cannot send data");
                        break;
                    }
                };
                match response.status {
                    gemini::protocol::StatusCode::Redirect(c) => {
                        println!("Redirect code {c} with meta {}", response.meta);
                        url = match Url::try_from(response.meta.as_str()) {
                            Ok(r) => r,
                            Err(e) => {
                                let estr = format!("{e:?}");
                                sender
                                    .send(scheme::Response::Error(estr))
                                    .expect("Cannot send data");
                                break;
                            }
                        };
                    }
                    gemini::protocol::StatusCode::Success(_) => {
                        let mime = if response.meta.starts_with("text/gemini") {
                            String::from("text/gemini")
                        } else if let Some((mime, _)) = response.meta.split_once(';') {
                            String::from(mime)
                        } else {
                            response.meta
                        };
                        let url = Some(url.to_string());
                        let content = scheme::Content {
                            url,
                            mime,
                            bytes: response.data,
                        };
                        sender
                            .send(scheme::Response::Success(content))
                            .expect("Cannot send data");
                        break;
                    }
                    gemini::protocol::StatusCode::Input(sensitive) => {
                        let input = scheme::Input {
                            meta: response.meta,
                            url: url.to_string(),
                            sensitive,
                        };
                        sender
                            .send(scheme::Response::RequestInput(input))
                            .expect("Cannot send data");
                        break;
                    }
                    s => {
                        let estr = format!("{s:?}");
                        sender
                            .send(scheme::Response::Error(estr))
                            .expect("Cannot send data");
                        break;
                    }
                }
            }
        });
        let viewer = self.clone();
        receiver.attach(None, move |response| {
            match response {
                scheme::Response::RequestInput(input) => {
                    let signal = if input.sensitive == 1 {
                        "request-input-sensitive"
                    } else {
                        "request-input"
                    };
                    viewer.append_history(&input.url);
                    viewer.emit_by_name::<()>(signal, &[&input.meta, &input.url]);
                }
                scheme::Response::Success(content) => {
                    viewer.process_gemini_response_success(&content, &url);
                }
                scheme::Response::Redirect(_s) => {}
                scheme::Response::Error(estr) => {
                    viewer.emit_by_name::<()>("page-load-failed", &[&estr]);
                }
            }
            Continue(false)
        });
    }

    fn process_gemini_response_success(&self, content: &Content, url: &Url) {
        self.set_buffer_mime(&content.mime);
        self.set_buffer_content(&content.bytes);
        let end_url = content.url.as_ref().unwrap();
        match content.mime.as_str() {
            "text/gemini" => {
                self.append_history(end_url);
                self.render_gmi(&String::from_utf8_lossy(&content.bytes));
                self.emit_by_name::<()>("page-loaded", &[end_url]);
            }
            s if s.starts_with("text/") => {
                self.append_history(end_url);
                self.render_text(&String::from_utf8_lossy(&content.bytes));
                self.emit_by_name::<()>("page-loaded", &[end_url]);
            }
            s if s.starts_with("image") => {
                self.append_history(end_url);
                self.render_image_from_bytes(&content.bytes);
                self.emit_by_name::<()>("page-loaded", &[end_url]);
            }
            _ => {
                let derived = tree_magic_mini::from_u8(&content.bytes);
                if derived.starts_with("text") {
                    self.append_history(end_url);
                    self.render_text(&String::from_utf8_lossy(&content.bytes));
                    self.emit_by_name::<()>("page-loaded", &[end_url]);
                } else if derived.starts_with("image") {
                    self.append_history(end_url);
                    self.render_image_from_bytes(&content.bytes);
                    self.emit_by_name::<()>("page-loaded", &[end_url]);
                } else {
                    let filename = if let Some(segments) = url.path_segments() {
                        segments.last().unwrap_or("download")
                    } else {
                        "download"
                    }
                    .to_string();
                    self.emit_by_name::<()>("request-download", &[&content.mime, &filename]);
                }
            }
        }
    }

    /// Reloads the current page
    pub fn reload(&self) {
        self.load(&self.uri());
    }

    /// Connects to the "page-load-started" signal, emitted when the browser
    /// begins loading a uri
    pub fn connect_page_load_started<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("page-load-started", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = values[1].get::<String>().unwrap();
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
            let uri = values[1].get::<String>().unwrap();
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
            let uri = values[1].get::<String>().unwrap();
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
            let uri = values[1].get::<String>().unwrap();
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

    /// Connects to the "request-download" signal, emitter when the browser has
    /// encountered a request for a file type it does not know how to render
    pub fn connect_request_download<F: Fn(&Self, String, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local(
            "request-download",
            true,
            move |values| -> Option<glib::Value> {
                let obj = values[0].get::<Self>().unwrap();
                let mime = values[1].get::<String>().unwrap();
                let filename = values[2].get::<String>().unwrap();
                f(&obj, mime, filename);
                None
            },
        )
    }

    /// Connects to the "request-new-tab" signal, emitted when the "Open in new
    /// tab" item is chosen from the context menu for link items.
    pub fn connect_request_new_tab<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-new-tab", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = values[1].get::<String>().unwrap();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "request-new-window" signal, emitted when the "Open in
    /// new window" item is chosen from the context menu for link items.
    pub fn connect_request_new_window<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-new-window", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = values[1].get::<String>().unwrap();
            f(&obj, uri);
            None
        })
    }

    /// Connects to the "request-input" signal, emitted when the server has
    /// requested input. The signal handler should repeat the page request with
    /// the user input appended.
    pub fn connect_request_input<F: Fn(&Self, String, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-input", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let meta = values[1].get::<String>().unwrap();
            let url = values[2].get::<String>().unwrap();
            f(&obj, meta, url);
            None
        })
    }

    /// Connects to the "request-input-sensitive" signal, emitted when the server
    /// has requested sensitive input. The signal handler should repeat the page
    /// request with the user input appended.
    pub fn connect_request_input_sensitive<F: Fn(&Self, String, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-input-sensitive", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let meta = values[1].get::<String>().unwrap();
            let url = values[2].get::<String>().unwrap();
            f(&obj, meta, url);
            None
        })
    }

    /// Connects to the "request-upload" signal, emitted when clicking on a
    /// Spartan protocol Prompt link
    pub fn connect_request_upload<F: Fn(&Self, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_local("request-upload", true, move |values| {
            let obj = values[0].get::<Self>().unwrap();
            let uri = values[1].get::<String>().unwrap();
            f(&obj, uri);
            None
        })
    }

    fn wrap_text(&self, text: &str, font_size: i32) -> String {
        let factor = font_size / 1525;
        let width: usize = match self.root() {
            Some(win) => std::cmp::min((win.width() / factor).try_into().unwrap(), 175),
            None => 175,
        };
        fill(glib::markup_escape_text(text).as_str(), width)
    }
}
