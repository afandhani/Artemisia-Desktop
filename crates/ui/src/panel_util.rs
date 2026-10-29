use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Switch};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub fn clear(b: &GtkBox) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
    }
}

pub fn dim(text: &str) -> Label {
    let l = Label::new(Some(text));
    l.set_xalign(0.0);
    l.add_css_class("cc-dim");
    l
}

/// Judul seksi: nama di kiri, saklar di kanan.
pub fn header(title: &str) -> (GtkBox, Switch) {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    let label = Label::new(Some(title));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.add_css_class("cc-title");
    let switch = Switch::new();
    switch.set_valign(Align::Center);
    row.append(&label);
    row.append(&switch);
    (row, switch)
}

/// Saklar yang dipengaruhi user dan service tanpa saling menimpa.
pub struct SwitchSync {
    switch: Switch,
        updating: Rc<Cell<bool>>,
        last_user: Rc<Cell<Option<Instant>>>,
}

impl SwitchSync {
    pub fn new(switch: &Switch, on_toggle: impl Fn(bool) + 'static) -> Self {
        let updating = Rc::new(Cell::new(false));
        let last_user: Rc<Cell<Option<Instant>>> = Rc::new(Cell::new(None));
        {
            let updating = updating.clone();
            let last_user = last_user.clone();
            switch.connect_active_notify(move |s| {
                if updating.get() {
                    return;
                }
                last_user.set(Some(Instant::now()));
                on_toggle(s.is_active());
            });
        }
        Self { switch: switch.clone(), updating, last_user }
    }

    /// Ikuti nilai dari service, kecuali user baru saja menyentuhnya.
    pub fn sync(&self, value: bool) {
        let touched = self
        .last_user
        .get()
        .map_or(false, |t| t.elapsed() < Duration::from_secs(3));
        if touched || self.switch.is_active() == value {
            return;
        }
        self.updating.set(true);
        self.switch.set_active(value);
        self.updating.set(false);
    }
}
