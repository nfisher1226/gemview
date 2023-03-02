use {
    gtk::{
        glib,
        glib::{subclass::Signal, Properties},
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

#[derive(Default, Properties)]
#[properties(wrapper_type = super::GemView)]
pub struct GemView {
    pub(crate) history: RefCell<History>,
    pub(crate) buffer: RefCell<Buffer>,
    #[property(get, set)]
    pub(crate) font_paragraph: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) font_pre: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) font_quote: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) font_h1: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) font_h2: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) font_h3: RefCell<FontDescription>,
    #[property(get, set)]
    pub(crate) paragraph_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h1_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h2_tag: RefCell<gtk::TextTag>,
    #[property(get, set)]
    pub(crate) h3_tag: RefCell<gtk::TextTag>,
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
    fn properties() -> &'static [glib::ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, _id: usize, _value: &glib::Value, _pspec: &glib::ParamSpec) {
        Self::derived_set_property(self, _id, _value, _pspec)
    }

    fn property(&self, _id: usize, _pspec: &glib::ParamSpec) -> glib::Value {
        Self::derived_property(self, _id, _pspec)
    }

    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_editable(false);
        obj.set_cursor_visible(false);
        *self.history.borrow_mut() = History::default();
        let buffer = obj.buffer();
        let mut font = FontDescription::new();
        font.set_family("Sans");
        font.set_style(Style::Normal);
        font.set_weight(Weight::Book);
        font.set_size(12);
        let normal = buffer
            .create_tag(
                Some("normal"),
                &[
                    ("font", &font.to_string()),
                    ("justification", &gtk::Justification::Fill),
                ],
            )
            .unwrap();
        obj.set_paragraph_tag(normal);
        obj.set_font_paragraph(&font);
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
        let h1tag = buffer
            .create_tag(
                Some("h1"),
                &[
                    ("font", &"Sans Normal 18"),
                    ("justification", &gtk::Justification::Fill),
                ],
            )
            .unwrap();
        *self.h1_tag.borrow_mut() = h1tag;
        let h2tag = buffer
            .create_tag(
                Some("h2"),
                &[
                    ("font", &"Sans Normal 16"),
                    ("justification", &gtk::Justification::Fill),
                ],
            )
            .unwrap();
        *self.h2_tag.borrow_mut() = h2tag;
        let h3tag = buffer
            .create_tag(
                Some("h3"),
                &[
                    ("font", &"Sans Normal 14"),
                    ("justification", &gtk::Justification::Fill),
                ],
            )
            .unwrap();
        *self.h3_tag.borrow_mut() = h3tag;
        obj.bind_properties();
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
