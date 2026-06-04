pub enum Status {
    Ok,
    Warning,
    Missing,
    Step,
}

impl Status {
    fn glyph(&self) -> &'static str {
        match self {
            Status::Ok => "✓",
            Status::Warning => "⚠",
            Status::Missing => "✗",
            Status::Step => "→",
        }
    }
}

pub fn line(status: Status, text: impl AsRef<str>) -> String {
    format!("{} {}", status.glyph(), text.as_ref())
}
