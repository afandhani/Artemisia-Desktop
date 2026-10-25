use gtk4::prelude::*;
use gtk4::Button;

pub fn button(on_click: impl Fn() + 'static) -> Button {
    let btn = Button::with_label("☰");
    btn.set_has_frame(false);
    btn.add_css_class("launcher");
    btn.connect_clicked(move |_| on_click());
    btn
}
