use gtk4::prelude::*;
use gtk4::{gdk, glib, Application, ApplicationWindow, CenterBox, CssProvider};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("../../../assets/style/main.css"));
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("tidak ada display"),
                                                 &provider,
                                                 gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
    .application_id("dev.artemisia.bar")
    .build();

    app.connect_activate(|app| {
        load_css();
        let cc = ui::control_center::ControlCenter::new(app);

        let win = ApplicationWindow::builder().application(app).build();
        win.add_css_class("bar");
        win.init_layer_shell();
        win.set_layer(Layer::Top);
        win.set_keyboard_mode(KeyboardMode::OnDemand);
        win.auto_exclusive_zone_enable();
        for edge in [Edge::Top, Edge::Left, Edge::Right] {
            win.set_anchor(edge, true);
        }

        let clock = ui::clock::widget();

        let left = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        left.append(&ui::launcher::button());
        left.append(&ui::tags::widget(mango::watch(), mango::view_tag));

        let status = ui::status::widget(
            &[
                ui::status::Item::Network,
                ui::status::Item::Volume,
                ui::status::Item::Battery,
            ],
            {
                let cc = cc.clone();
                move || cc.toggle()
            },
        );

        let bar = CenterBox::new();
        bar.add_css_class("bar-inner");
        bar.set_start_widget(Some(&left));
        bar.set_center_widget(Some(&clock));
        bar.set_end_widget(Some(&status));

        win.set_child(Some(&bar));
        win.present();
    });

    app.run()
}


