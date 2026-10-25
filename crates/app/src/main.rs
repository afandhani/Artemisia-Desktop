use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Label};
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn main() -> glib::ExitCode {
    let app = Application::builder()
    .application_id("dev.artemisia.bar")
    .build();
    app.connect_activate(|app| {
        let win = ApplicationWindow::builder().application(app).build();
        win.init_layer_shell();
        win.set_layer(Layer::Top);
        win.auto_exclusive_zone_enable();
        for edge in [Edge::Top, Edge::Left, Edge::Right] {
            win.set_anchor(edge, true);
        }
        win.set_child(Some(&Label::new(Some("Artemisia"))));
        win.present();
    });
    app.run()
}
