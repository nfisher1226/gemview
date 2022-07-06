mod imp;

use gtk::{
    glib::{self, Object},
    prelude::*,
    subclass::prelude::*,
};

glib::wrapper! {
    pub struct UploadWidget(ObjectSubclass<imp::UploadWidget>)
        @extends gtk::Box, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
            gtk::Orientable;
}

impl Default for UploadWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl UploadWidget {
    pub fn new() -> Self {
        Object::new(&[("orientation", &gtk::Orientation::Vertical)])
            .expect("Cannot create upload widget")
    }
}
