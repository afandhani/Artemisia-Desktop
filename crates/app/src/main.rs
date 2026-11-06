use std::{rc::Rc, time::Duration};

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use artemisia_config_watcher::ConfigWatcher;
use artemisia_core::{Position, ShellPlan};

fn load_css() {
    let provider = CssProvider::new();

    provider.load_from_data(include_str!("../../../assets/style/main.css"));

    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Tidak ada display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn apply_position(win: &ApplicationWindow, position: Position) {
    match position {
        Position::Top => {
            win.set_anchor(Edge::Top, true);
            win.set_anchor(Edge::Bottom, false);
        }
        Position::Bottom => {
            win.set_anchor(Edge::Top, false);
            win.set_anchor(Edge::Bottom, true);
        }
    }
}

fn main() -> glib::ExitCode {
    let config = match config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("\n[Artemisia] Failed to load configuration:\n{error}\n");
            std::process::exit(1);
        }
    };

    let shell_plan = ShellPlan::from(&config);

    let app = Application::builder()
        .application_id("dev.artemisia.bar")
        .build();

    app.connect_activate(move |app| {
        load_css();

        // Control Center
        let cc = ui::control_center::ControlCenter::new(app);
        let cc_toggle = cc.clone();

        let on_status_click: Rc<dyn Fn()> = Rc::new(move || cc_toggle.toggle());

        // Initial panel
        let panel = ui::panel::build(&shell_plan, on_status_click.clone());

        // Panel window
        let win = ApplicationWindow::builder().application(app).build();

        win.add_css_class("bar");

        win.init_layer_shell();
        win.set_layer(Layer::Top);
        win.set_keyboard_mode(KeyboardMode::OnDemand);
        win.auto_exclusive_zone_enable();

        win.set_anchor(Edge::Left, true);
        win.set_anchor(Edge::Right, true);

        apply_position(&win, shell_plan.position);

        win.set_child(Some(&panel));
        win.present();

        // Initialize config watcher
        let mut watcher = match ConfigWatcher::new() {
            Ok(watcher) => watcher,
            Err(error) => {
                eprintln!("[Artemisia] Failed to initialize config watcher: {error}");
                return;
            }
        };

        println!("[Artemisia] Config watcher initialized.");

        // References for the GTK main loop
        let win_clone = win.clone();
        let status_callback = on_status_click.clone();

        // Live configuration reload
        glib::timeout_add_local(Duration::from_millis(50), move || {
            match watcher.poll() {
                Ok(Some(new_config)) => {
                    println!("[Artemisia] Configuration changed!");

                    let new_plan = ShellPlan::from(&new_config);

                    println!("[Artemisia] New plan: {new_plan:#?}");

                    let new_panel = ui::panel::build(&new_plan, status_callback.clone());

                    apply_position(&win_clone, new_plan.position);

                    win_clone.set_child(Some(&new_panel));

                    println!("[Artemisia] Panel rebuilt!");
                }

                Ok(None) => {}

                Err(error) => {
                    eprintln!("[Artemisia] Configuration reload failed: {error}");
                }
            }

            glib::ControlFlow::Continue
        });
    });

    app.run()
}
