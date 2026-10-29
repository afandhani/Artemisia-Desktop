use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, GestureClick, Label, Orientation};
use mango::{Receiver, TagsState};
use std::rc::Rc;

pub fn widget(rx: Receiver<TagsState>, on_click: impl Fn(u8) + 'static) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 4);
    container.add_css_class("tags");

    let on_click: Rc<dyn Fn(u8)> = Rc::new(on_click);
    let c = container.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = rx.recv().await {
            render(&c, &state, &on_click);
        }
    });

    container
}

const PINNED: u8 = 0;

fn render(container: &GtkBox, state: &TagsState, on_click: &Rc<dyn Fn(u8)>) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    for tag in &state.tags {
        if !tag.occupied && !tag.active && !tag.urgent && tag.index > PINNED {
            continue;
        }
        let label = Label::new(Some(&tag.index.to_string()));
        label.add_css_class("tag");
        if tag.occupied {
            label.add_css_class("occupied");
        }
        if tag.active {
            label.add_css_class("active");
        }
        if tag.urgent {
            label.add_css_class("urgent");
        }

        let index = tag.index;
        let cb = on_click.clone();
        let gesture = GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| cb(index));
        label.add_controller(gesture);

        container.append(&label);
    }
}
