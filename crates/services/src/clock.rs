use chrono::Local;

pub fn time_text() -> String {
    time_text_with_format("%H:%M")
}

pub fn time_text_with_format(format: &str) -> String {
    Local::now().format(format).to_string()
}
