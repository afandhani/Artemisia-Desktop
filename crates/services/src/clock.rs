use chrono::Local;

pub fn time_text() -> String {
    Local::now().format("%H:%M").to_string()
}
