use gtk4::gio::{self, prelude::*, AppInfo};
use gtk4::prelude::*;
use gtk4::{
    ArrowType, Box as GtkBox, Button, Image, Label, MenuButton, Orientation, Popover,
    ScrolledWindow,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

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

// Ubah urutan/aturan di sini kalau ingin pengelompokan lain.
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

pub fn button() -> MenuButton {
    let menu = MenuButton::new();
    menu.set_label("☰");
    menu.set_has_frame(false);
    menu.add_css_class("launcher");

    let popover = Popover::new();
    popover.set_has_arrow(false);
    let list = GtkBox::new(Orientation::Vertical, 2);
    popover.set_child(Some(&list));
    menu.set_popover(Some(&popover));

    // Bangun ulang tiap menu dibuka, supaya daftar aplikasi selalu segar.
    popover.connect_show(move |p| fill(&list, p));

    menu
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

fn fill(list: &GtkBox, main: &Popover) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }

    let mut groups: BTreeMap<Group, Vec<AppInfo>> = BTreeMap::new();
    for app in AppInfo::all() {
        if !app.should_show() {
            continue;
        }
        let categories = categories_of(&app);
        groups.entry(group_for(&categories)).or_default().push(app);
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
        if let Err(e) = app.launch(&[], None::<&gio::AppLaunchContext>) {
            eprintln!("launcher: gagal membuka {}: {e}", app.name());
        }
        if let Some(p) = main.upgrade() {
            p.popdown();
        }
    });

    btn
}
