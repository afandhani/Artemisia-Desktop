use gtk4::prelude::*;
use gtk4::{glib, Label};

pub fn widget() -> Label {
    let label = Label::new(Some(&services::clock::time_text()));
    label.add_css_class("clock");

    let l = label.clone();
    glib::timeout_add_seconds_local(1, move || {
        l.set_text(&services::clock::time_text());
        glib::ControlFlow::Continue
    });

    label
}
