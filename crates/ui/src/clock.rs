use gtk4::prelude::*;
use gtk4::{Label, glib};

pub fn widget() -> Label {
    widget_with_format("%H:%M")
}

pub fn widget_with_format(format: impl Into<String>) -> Label {
    let format = format.into();

    let label = Label::new(Some(&services::clock::time_text_with_format(&format)));

    label.add_css_class("clock");

    let label_clone = label.clone();
    let update_format = format.clone();

    glib::timeout_add_seconds_local(1, move || {
        label_clone.set_text(&services::clock::time_text_with_format(&update_format));

        glib::ControlFlow::Continue
    });

    label
}
