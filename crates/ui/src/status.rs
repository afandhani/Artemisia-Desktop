use gtk4::prelude::*;
use gtk4::{glib, Box as GtkBox, GestureClick, Label, Orientation};
use services::network::NetworkState;
use services::{battery, network, volume, Receiver};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Item {
    Battery,
    Volume,
    Network,
}

pub fn widget(items: &[Item], on_click: impl Fn() + 'static) -> GtkBox {
    let container = GtkBox::new(Orientation::Horizontal, 10);
    container.add_css_class("status");

    for item in items {
        let label = match item {
            Item::Battery => bind(battery::watch(), |s| {
                format!("BAT {}%{}", s.percent, if s.charging { "+" } else { "" })
            }),
            Item::Volume => bind(volume::watch(), |s| {
                if s.muted {
                    "VOL mute".to_string()
                } else {
                    format!("VOL {}%", s.percent)
                }
            }),
            Item::Network => bind(network::watch(), |s| match s {
                NetworkState::Wifi { ssid, signal } => format!("WIFI {ssid} {signal}%"),
                                  NetworkState::Disconnected => "WIFI off".to_string(),
            }),
        };
        container.append(&label);
    }

    let gesture = GestureClick::new();
    gesture.connect_released(move |_, _, _, _| on_click());
    container.add_controller(gesture);

    container
}

fn bind<T: 'static>(rx: Receiver<T>, to_text: impl Fn(&T) -> String + 'static) -> Label {
    let label = Label::new(None);
    label.add_css_class("status-item");

    let l = label.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok(state) = rx.recv().await {
            l.set_text(&to_text(&state));
        }
    });

    label
}
