use crate::panel_util::{clear, dim, header, SwitchSync};
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Entry, Label, Orientation, ScrolledWindow};
use services::network::{self, WifiList};
use std::cell::RefCell;
use std::rc::Rc;

struct Prompt {
    row: GtkBox,
    label: Label,
    entry: Entry,
    pending: RefCell<Option<String>>,
}

impl Prompt {
    fn ask(&self, ssid: &str) {
        self.label.set_text(&format!("Password untuk {ssid}"));
        self.entry.set_text("");
        *self.pending.borrow_mut() = Some(ssid.to_string());
        self.row.set_visible(true);
        self.entry.grab_focus();
    }

    fn submit(&self) {
        let ssid = self.pending.borrow_mut().take();
        if let Some(ssid) = ssid {
            let pw = self.entry.text().to_string();
            if !pw.is_empty() {
                network::connect(&ssid, Some(pw.as_str()));
            }
        }
        self.entry.set_text("");
        self.row.set_visible(false);
    }
}

pub fn widget() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 6);
    let (head, switch) = header("Wi-Fi");
    let sync = SwitchSync::new(&switch, network::set_wifi_enabled);

    let list = GtkBox::new(Orientation::Vertical, 2);
    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&list));
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(200);

    // Kolom password untuk jaringan terkunci yang belum tersimpan.
    let prompt_box = GtkBox::new(Orientation::Vertical, 4);
    let prompt_label = Label::new(None);
    prompt_label.set_xalign(0.0);
    let inner = GtkBox::new(Orientation::Horizontal, 6);
    let entry = Entry::new();
    entry.set_visibility(false);
    entry.set_hexpand(true);
    let ok = Button::with_label("Hubungkan");
    inner.append(&entry);
    inner.append(&ok);
    prompt_box.append(&prompt_label);
    prompt_box.append(&inner);
    prompt_box.set_visible(false);

    let prompt = Rc::new(Prompt {
        row: prompt_box.clone(),
                         label: prompt_label,
                         entry: entry.clone(),
                         pending: RefCell::new(None),
    });
    {
        let p = prompt.clone();
        ok.connect_clicked(move |_| p.submit());
    }
    {
        let p = prompt.clone();
        entry.connect_activate(move |_| p.submit());
    }

    root.append(&head);
    root.append(&scroll);
    root.append(&prompt_box);

    let rx = network::watch_list();
    {
        let list = list.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(state) = rx.recv().await {
                sync.sync(state.enabled);
                render(&list, &state, &prompt);
            }
        });
    }

    root
}

fn render(list: &GtkBox, state: &WifiList, prompt: &Rc<Prompt>) {
    clear(list);
    if !state.enabled {
        list.append(&dim("Wi-Fi mati"));
        return;
    }
    if state.aps.is_empty() {
        list.append(&dim("Tidak ada jaringan"));
        return;
    }

    for ap in &state.aps {
        let row = GtkBox::new(Orientation::Horizontal, 8);
        let name = Label::new(Some(ap.ssid.as_str()));
        name.set_xalign(0.0);
        name.set_hexpand(true);
        let info = Label::new(Some(&format!(
            "{}%{}",
            ap.signal,
            if ap.secured { " [lock]" } else { "" }
        )));
        row.append(&name);
        row.append(&info);

        let btn = Button::new();
        btn.set_child(Some(&row));
        btn.set_has_frame(false);
        btn.add_css_class("wifi-ap");
        if ap.active {
            btn.add_css_class("active");
        }

        let ssid = ap.ssid.clone();
        let secured = ap.secured;
        let active = ap.active;
        let saved = state.saved.contains(&ap.ssid);
        let prompt = prompt.clone();
        btn.connect_clicked(move |_| {
            if active {
                network::disconnect(&ssid);
            } else if saved {
                network::connect_saved(&ssid);
            } else if secured {
                prompt.ask(&ssid);
            } else {
                network::connect(&ssid, None);
            }
        });

        list.append(&btn);
    }
}
