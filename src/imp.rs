use gtk::glib;
use glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::pango::{FontDescription, Style, Weight};
use once_cell::sync::Lazy;
use std::cell::RefCell;


#[derive(Default)]
pub struct GemView {
    pub uri: RefCell<String>,
    pub font_paragraph: RefCell<FontDescription>,
    pub font_pre: RefCell<FontDescription>,
    pub font_quote: RefCell<FontDescription>,
    pub font_h1: RefCell<FontDescription>,
    pub font_h2: RefCell<FontDescription>,
    pub font_h3: RefCell<FontDescription>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for GemView {
    const NAME: &'static str = "GemView";
    type Type = super::GemView;
    type ParentType = gtk::TextView;
}

// Trait shared by all GObjects
impl ObjectImpl for GemView {
    fn constructed(&self, obj: &Self::Type) {
        self.parent_constructed(obj);
        obj.set_editable(false);
        obj.set_cursor_visible(false);
        *self.uri.borrow_mut() = String::from("about:blank");
        let mut font = FontDescription::new();
        font.set_family("Sans");
        font.set_style(Style::Normal);
        font.set_weight(Weight::Book);
        font.set_size(12);
        *self.font_paragraph.borrow_mut() = font.clone();
        *self.font_quote.borrow_mut() = font.clone();
        font.set_family("Monospace");
        *self.font_pre.borrow_mut() = font.clone();
        font.set_family("Sans");
        font.set_weight(Weight::Medium);
        font.set_size(14);
        *self.font_h3.borrow_mut() = font.clone();
        font.set_weight(Weight::Bold);
        font.set_size(16);
        *self.font_h2.borrow_mut() = font.clone();
        font.set_weight(Weight::Heavy);
        font.set_size(18);
        *self.font_h1.borrow_mut() = font
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                "page-loaded",
                &[String::static_type().into()],
                <()>::static_type().into(),
            ).build(),
            Signal::builder(
                "page-load-started",
                &[String::static_type().into()],
                <()>::static_type().into(),
            ).build(),
            Signal::builder(
                "page-load-redirect",
                &[String::static_type().into()],
                <()>::static_type().into(),
            ).build(),
            Signal::builder(
                "page-load-failed",
                &[String::static_type().into()],
                <()>::static_type().into(),
            ).build()]
        });
        SIGNALS.as_ref()
    }
}

// Trait shared by all widgets
impl WidgetImpl for GemView {}

impl TextViewImpl for GemView {}
