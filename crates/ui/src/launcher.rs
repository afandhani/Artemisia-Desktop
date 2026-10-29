use gtk4::gio::{self, prelude::*, AppInfo};
use gtk4::prelude::*;
use gtk4::{
    ArrowType, Box as GtkBox, Button, Entry, Image, Label, MenuButton, Orientation, Popover,
    ScrolledWindow,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Group {
    Games,
    Multimedia,
    Development,
    Graphics,
    Internet,
    Office,
    Utilities,
    System,
    Other,
}

impl Group {
    fn label(self) -> &'static str {
        match self {
            Group::Games => "Games",
            Group::Multimedia => "Multimedia",
            Group::Development => "Development",
            Group::Graphics => "Graphics",
            Group::Internet => "Internet",
            Group::Office => "Office",
            Group::Utilities => "Utilities",
            Group::System => "System",
            Group::Other => "Other",
        }
    }
}

fn group_for(categories: &str) -> Group {
    let has = |c: &str| categories.split(';').any(|x| x == c);
    if has("Game") {
        Group::Games
    } else if has("AudioVideo") || has("Audio") || has("Video") {
        Group::Multimedia
    } else if has("Development") {
        Group::Development
    } else if has("Graphics") {
        Group::Graphics
    } else if has("Network") {
        Group::Internet
    } else if has("Office") {
        Group::Office
    } else if has("Settings") || has("System") {
        Group::System
    } else if has("Utility") {
        Group::Utilities
    } else {
        Group::Other
    }
}

fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    match std::env::var("XDG_DATA_HOME") {
        Ok(p) if !p.is_empty() => dirs.push(PathBuf::from(p)),
        _ => {
            if let Ok(home) = std::env::var("HOME") {
                dirs.push(PathBuf::from(home).join(".local/share"));
            }
        }
    }
    let sys = std::env::var("XDG_DATA_DIRS").unwrap_or_default();
    let sys = if sys.is_empty() {
        "/usr/local/share:/usr/share".to_string()
    } else {
        sys
    };
    dirs.extend(sys.split(':').map(PathBuf::from));
    dirs
}

// Baca baris "Categories=" dari file .desktop aplikasi.
fn categories_of(app: &AppInfo) -> String {
    let Some(id) = app.id() else {
        return String::new();
    };
    for dir in data_dirs() {
        let path = dir.join("applications").join(id.as_str());
        if let Ok(text) = std::fs::read_to_string(path) {
            return text
            .lines()
            .find_map(|l| l.strip_prefix("Categories="))
            .unwrap_or("")
            .to_string();
        }
    }
    String::new()
}

fn visible_apps() -> Vec<AppInfo> {
    AppInfo::all()
    .into_iter()
    .filter(|a| a.should_show())
    .collect()
}

fn clear(b: &GtkBox) {
    while let Some(child) = b.first_child() {
        b.remove(&child);
    }
}

fn launch(app: &AppInfo) {
    if let Err(e) = app.launch(&[], None::<&gio::AppLaunchContext>) {
        eprintln!("launcher: gagal membuka {}: {e}", app.name());
    }
}

// Hasil pencarian: nama yang diawali kata kunci dulu, lalu yang mengandungnya.
fn search(apps: &[AppInfo], query: &str) -> Vec<AppInfo> {
    let mut hits: Vec<(bool, String, AppInfo)> = apps
    .iter()
    .filter_map(|a| {
        let name = a.name().to_lowercase();
        name.contains(query)
        .then(|| (!name.starts_with(query), name, a.clone()))
    })
    .collect();
    hits.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    hits.into_iter().take(8).map(|(_, _, a)| a).collect()
}

pub fn button() -> MenuButton {
    let menu = MenuButton::new();
    menu.set_label("☰");
    menu.set_has_frame(false);
    menu.add_css_class("launcher");

    let popover = Popover::new();
    popover.set_has_arrow(false);

    let root = GtkBox::new(Orientation::Vertical, 6);
    let categories = GtkBox::new(Orientation::Vertical, 2);
    let results = GtkBox::new(Orientation::Vertical, 2);
    results.set_visible(false);

    let entry = Entry::new();
    entry.set_placeholder_text(Some("Cari aplikasi..."));
    entry.add_css_class("launcher-search");

    root.append(&categories);
    root.append(&results);
    root.append(&entry);
    popover.set_child(Some(&root));
    menu.set_popover(Some(&popover));

    // Daftar aplikasi dibaca ulang tiap menu dibuka.
    let apps: Rc<RefCell<Vec<AppInfo>>> = Rc::new(RefCell::new(Vec::new()));

    {
        let apps = apps.clone();
        let categories = categories.clone();
        let results = results.clone();
        let entry = entry.clone();
        popover.connect_show(move |p| {
            entry.set_text("");
            *apps.borrow_mut() = visible_apps();
            fill_categories(&categories, &apps.borrow(), p);
            results.set_visible(false);
            categories.set_visible(true);
            entry.grab_focus();
        });
    }

    {
        let apps = apps.clone();
        let categories = categories.clone();
        let results = results.clone();
        let main = popover.downgrade();
        entry.connect_changed(move |e| {
            let query = e.text().to_lowercase();
            clear(&results);
            if query.is_empty() {
                results.set_visible(false);
                categories.set_visible(true);
                return;
            }
            categories.set_visible(false);
            results.set_visible(true);
            let Some(main) = main.upgrade() else { return };
            for app in search(&apps.borrow(), &query) {
                results.append(&app_button(&app, &main));
            }
        });
    }

    {
        // Enter: buka hasil pertama.
        let apps = apps.clone();
        let main = popover.downgrade();
        entry.connect_activate(move |e| {
            let query = e.text().to_lowercase();
            if query.is_empty() {
                return;
            }
            if let Some(app) = search(&apps.borrow(), &query).first() {
                launch(app);
                if let Some(p) = main.upgrade() {
                    p.popdown();
                }
            }
        });
    }

    menu
}

fn fill_categories(list: &GtkBox, apps: &[AppInfo], main: &Popover) {
    clear(list);

    let mut groups: BTreeMap<Group, Vec<AppInfo>> = BTreeMap::new();
    for app in apps {
        let categories = categories_of(app);
        groups
        .entry(group_for(&categories))
        .or_default()
        .push(app.clone());
    }

    for (group, mut apps) in groups {
        apps.sort_by_key(|a| a.name().to_lowercase());
        list.append(&category_button(group, &apps, main));
    }
}

fn category_button(group: Group, apps: &[AppInfo], main: &Popover) -> MenuButton {
    let btn = MenuButton::new();
    btn.set_direction(ArrowType::Right);
    btn.set_has_frame(false);
    btn.add_css_class("launcher-category");
    btn.set_label(&format!("{} ({})", group.label(), apps.len()));

    let apps_box = GtkBox::new(Orientation::Vertical, 2);
    for app in apps {
        apps_box.append(&app_button(app, main));
    }

    let scroll = ScrolledWindow::new();
    scroll.set_child(Some(&apps_box));
    scroll.set_propagate_natural_height(true);
    scroll.set_max_content_height(420);
    scroll.set_min_content_width(240);

    let pop = Popover::new();
    pop.set_has_arrow(false);
    pop.set_child(Some(&scroll));
    btn.set_popover(Some(&pop));

    btn
}

fn app_button(app: &AppInfo, main: &Popover) -> Button {
    let row = GtkBox::new(Orientation::Horizontal, 8);
    if let Some(icon) = app.icon() {
        let img = Image::from_gicon(&icon);
        img.set_pixel_size(20);
        row.append(&img);
    }
    let name = Label::new(Some(app.name().as_str()));
    name.set_xalign(0.0);
    row.append(&name);

    let btn = Button::new();
    btn.set_child(Some(&row));
    btn.set_has_frame(false);
    btn.add_css_class("launcher-app");

    let app = app.clone();
    let main = main.downgrade();
    btn.connect_clicked(move |_| {
        launch(&app);
        if let Some(p) = main.upgrade() {
            p.popdown();
        }
    });

    btn
}
