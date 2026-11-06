use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use artemisia_core::Position;

fn load_css() {
    let provider = CssProvider::new();

    provider.load_from_data(include_str!("../../../assets/style/main.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Tidak ada display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() -> glib::ExitCode {
    // 1. Load configuration
    let config = match config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("\n[Artemisia] Failed to load configuration:\n{error}\n");

            std::process::exit(1);
        }
    };

    // 2. Convert config into ShellPlan
    let plan = Rc::new(artemisia_core::ShellPlan::from(&config));

    // 3. Create GTK application
    let app = Application::builder()
        .application_id("dev.artemisia.bar")
        .build();

    let shell_plan = artemisia_core::ShellPlan::from(&config);

    app.connect_activate(move |app| {
        load_css();

        // Control Center
        let cc = ui::control_center::ControlCenter::new(app);

        let cc_toggle = cc.clone();

        let on_status_click: Rc<dyn Fn()> = Rc::new(move || cc_toggle.toggle());

        // Build panel from ShellPlan
        let panel = ui::panel::build(&shell_plan, on_status_click);

        // Create panel window
        let win = ApplicationWindow::builder().application(app).build();

        win.add_css_class("bar");

        win.init_layer_shell();
        win.set_layer(Layer::Top);
        win.set_keyboard_mode(KeyboardMode::OnDemand);
        win.auto_exclusive_zone_enable();

        // Horizontal anchors
        win.set_anchor(Edge::Left, true);
        win.set_anchor(Edge::Right, true);

        // Vertical position from config
        match shell_plan.position {
            Position::Top => {
                win.set_anchor(Edge::Top, true);
                win.set_anchor(Edge::Bottom, false);
            }

            Position::Bottom => {
                win.set_anchor(Edge::Top, false);
                win.set_anchor(Edge::Bottom, true);
            }
        }

        win.set_child(Some(&panel));
        win.present();
    });

    app.run()
}
