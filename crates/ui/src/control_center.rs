use gtk4::prelude::*;
use gtk4::{
    glib, Application, ApplicationWindow, Box as GtkBox, Button, GestureClick, Label,
    Orientation, Scale, Separator,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use services::{brightness, volume, Receiver};
use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct ControlCenter {
    panel: ApplicationWindow,
    catcher: ApplicationWindow,
}

impl ControlCenter {
    pub fn new(app: &Application) -> Self {
        // Penangkap klik: jendela transparan layar penuh di belakang panel.
        let catcher = ApplicationWindow::builder().application(app).build();
        catcher.add_css_class("catcher");
        catcher.init_layer_shell();
        catcher.set_layer(Layer::Top);
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            catcher.set_anchor(edge, true);
        }

        // Panel: pojok kanan atas, di bawah bar.
        let panel = ApplicationWindow::builder().application(app).build();
        panel.add_css_class("control-center");
        panel.init_layer_shell();
        panel.set_layer(Layer::Overlay);
        panel.set_anchor(Edge::Top, true);
        panel.set_anchor(Edge::Right, true);
        panel.set_margin(Edge::Top, 6);
        panel.set_margin(Edge::Right, 8);
        panel.set_default_size(320, -1);
        panel.set_child(Some(&content()));
        panel.set_keyboard_mode(KeyboardMode::OnDemand);
        panel.connect_show(|_| services::network::rescan());

        let click = GestureClick::new();
        {
            let panel = panel.clone();
            let catcher = catcher.clone();
            click.connect_pressed(move |_, _, _, _| {
                panel.set_visible(false);
                catcher.set_visible(false);
            });
        }
        catcher.add_controller(click);

        Self { panel, catcher }
    }

    pub fn toggle(&self) {
        if self.panel.is_visible() {
            self.panel.set_visible(false);
            self.catcher.set_visible(false);
        } else {
            self.catcher.present();
            self.panel.present();
        }
    }
}

fn content() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 12);
    root.add_css_class("cc-box");

    let title = Label::new(Some("Control Center"));
    title.set_xalign(0.0);
    title.add_css_class("cc-title");
    root.append(&title);

    root.append(&slider_row(
        "Volume",
        0.0,
        volume::watch(),
                            |s| s.percent.min(100),
                            volume::set,
    ));

    let mute = Button::with_label("Mute / Unmute");
    mute.connect_clicked(|_| volume::toggle_mute());
    root.append(&mute);

    root.append(&slider_row(
        "Brightness",
        1.0,
        brightness::watch(),
                            |s| s.percent,
                            brightness::set,
    ));

    root.append(&Separator::new(Orientation::Horizontal));
    root.append(&crate::wifi_panel::widget());
    root.append(&Separator::new(Orientation::Horizontal));
    root.append(&crate::bluetooth_panel::widget());

    root
}

fn slider_row<T: 'static>(
    title: &str,
    min: f64,
    rx: Receiver<T>,
    to_percent: impl Fn(&T) -> u8 + 'static,
                          on_change: impl Fn(u8) + 'static,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Vertical, 4);
    let label = Label::new(Some(title));
    label.set_xalign(0.0);
    let scale = Scale::with_range(Orientation::Horizontal, min, 100.0, 1.0);
    scale.set_hexpand(true);
    scale.set_draw_value(true);
    row.append(&label);
    row.append(&scale);

    // updating: tandai perubahan dari service (bukan dari user).
    // last_user: abaikan update service sebentar setelah user menggeser.
    let updating = Rc::new(Cell::new(false));
    let last_user: Rc<Cell<Option<Instant>>> = Rc::new(Cell::new(None));

    {
        let updating = updating.clone();
        let last_user = last_user.clone();
        scale.connect_value_changed(move |s| {
            if updating.get() {
                return;
            }
            last_user.set(Some(Instant::now()));
            on_change(s.value().round() as u8);
        });
    }

    {
        let scale = scale.clone();
        glib::MainContext::default().spawn_local(async move {
            while let Ok(state) = rx.recv().await {
                let recently_touched = last_user
                .get()
                .map_or(false, |t| t.elapsed() < Duration::from_millis(1500));
                if recently_touched {
                    continue;
                }
                updating.set(true);
                scale.set_value(to_percent(&state) as f64);
                updating.set(false);
            }
        });
    }

    row
}
