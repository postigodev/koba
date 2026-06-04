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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_status_lines_with_centralized_glyphs() {
        assert_eq!(line(Status::Ok, "present"), "✓ present");
        assert_eq!(line(Status::Warning, "check config"), "⚠ check config");
        assert_eq!(line(Status::Missing, "missing"), "✗ missing");
        assert_eq!(line(Status::Step, "next"), "→ next");
    }
}
