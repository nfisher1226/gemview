use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;


#[derive(Default)]
pub struct GemView {
    pub uri: RefCell<String>
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
        *self.uri.borrow_mut() = String::from("about:blank");
    }
}

// Trait shared by all widgets
impl WidgetImpl for GemView {}

impl TextViewImpl for GemView {}
