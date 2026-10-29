use crate::panel_util::{clear, dim, header, SwitchSync};
use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, Button, Label, Orientation, ScrolledWindow};
use services::bluetooth::{self, BtState};

pub fn widget() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 6);
    let (head, switch) = header("Bluetooth");
    let sync = SwitchSync::new(&switch, bluetooth::set_powered);

    let list = GtkBox::new(Orientation::Vertical, 2);
    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&list));
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(200);

    root.append(&head);
    root.append(&scroll);

    let rx = bluetooth::watch();
    {
        let list = list.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(state) = rx.recv().await {
                sync.sync(state.powered);
                render(&list, &state);
            }
        });
    }

    root
}

fn render(list: &GtkBox, state: &BtState) {
    clear(list);
    if !state.available {
        list.append(&dim("Bluetooth tidak tersedia"));
        return;
    }
    if !state.powered {
        list.append(&dim("Bluetooth mati"));
        return;
    }
    if state.devices.is_empty() {
        list.append(&dim("Belum ada perangkat terpasang"));
        return;
    }

    for d in &state.devices {
        let row = GtkBox::new(Orientation::Horizontal, 8);
        let name = Label::new(Some(d.name.as_str()));
        name.set_xalign(0.0);
        name.set_hexpand(true);
        row.append(&name);
        if d.connected {
            row.append(&Label::new(Some("terhubung")));
        }

        let btn = Button::new();
        btn.set_child(Some(&row));
        btn.set_has_frame(false);
        btn.add_css_class("bt-device");
        if d.connected {
            btn.add_css_class("connected");
        }

        let mac = d.mac.clone();
        let connected = d.connected;
        btn.connect_clicked(move |_| {
            if connected {
                bluetooth::disconnect(&mac);
            } else {
                bluetooth::connect(&mac);
            }
        });

        list.append(&btn);
    }
}
