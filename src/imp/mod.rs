use {
    gtk::{
        glib,
        glib::subclass::Signal,
        pango::{FontDescription, Style, Weight},
        prelude::*,
        subclass::prelude::*,
    },
    once_cell::sync::Lazy,
    std::cell::RefCell,
};

mod buffer;
pub use buffer::Buffer;
mod history;
pub(crate) use history::History;

#[derive(Default)]
pub struct GemView {
    pub(crate) history: RefCell<History>,
    pub(crate) buffer: RefCell<Buffer>,
    pub(crate) font_paragraph: RefCell<FontDescription>,
    pub(crate) font_pre: RefCell<FontDescription>,
    pub(crate) font_quote: RefCell<FontDescription>,
    pub(crate) font_h1: RefCell<FontDescription>,
    pub(crate) font_h2: RefCell<FontDescription>,
    pub(crate) font_h3: RefCell<FontDescription>,
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
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_editable(false);
        obj.set_cursor_visible(false);
        *self.history.borrow_mut() = History::default();
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
        *self.font_h1.borrow_mut() = font;
        obj.add_actions();
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![
                Signal::builder("page-loaded")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-started")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-redirect")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("page-load-failed")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-unsupported-scheme")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-download")
                    .param_types([glib::Type::STRING, glib::Type::STRING])
                    .build(),
                Signal::builder("request-new-tab")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-new-window")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-input")
                    .param_types([glib::Type::STRING, glib::Type::STRING])
                    .build(),
                Signal::builder("request-input-sensitive")
                    .param_types([glib::Type::STRING])
                    .build(),
                Signal::builder("request-upload")
                    .param_types([glib::Type::STRING])
                    .build(),
            ]
        });
        SIGNALS.as_ref()
    }
}

// Trait shared by all widgets
impl WidgetImpl for GemView {}

impl TextViewImpl for GemView {}
