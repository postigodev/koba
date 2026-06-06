use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ok,
    Warn,
    Miss,
    Error,
    Run,
    Pass,
    Fail,
    Skip,
    Plan,
    Write,
    Keep,
    Refuse,
}

impl Status {
    pub fn badge(self) -> &'static str {
        match self {
            Status::Ok => "[ok]",
            Status::Warn => "[warn]",
            Status::Miss => "[miss]",
            Status::Error => "[error]",
            Status::Run => "[run]",
            Status::Pass => "[pass]",
            Status::Fail => "[fail]",
            Status::Skip => "[skip]",
            Status::Plan => "[plan]",
            Status::Write => "[write]",
            Status::Keep => "[keep]",
            Status::Refuse => "[refuse]",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusRow {
    status: Status,
    label: String,
    value: Option<String>,
    details: Vec<String>,
}

impl StatusRow {
    pub fn new(status: Status, label: impl Into<String>) -> Self {
        Self {
            status,
            label: label.into(),
            value: None,
            details: Vec::new(),
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }

    fn render(&self, label_width: usize) -> String {
        let mut output = String::new();
        match &self.value {
            Some(value) => {
                writeln!(
                    output,
                    "  {:<9}{:<label_width$}  {}",
                    self.status.badge(),
                    self.label,
                    value
                )
                .unwrap();
            }
            None => {
                writeln!(output, "  {:<9}{}", self.status.badge(), self.label).unwrap();
            }
        }

        for detail in &self.details {
            for line in detail.lines() {
                writeln!(output, "          {line}").unwrap();
            }
        }

        output
    }
}

pub fn row(status: Status, label: impl Into<String>) -> StatusRow {
    StatusRow::new(status, label)
}

pub fn line(status: Status, text: impl AsRef<str>) -> String {
    row(status, text.as_ref()).render(0).trim_end().to_owned()
}

pub fn render_rows(rows: &[StatusRow]) -> String {
    let label_width = rows
        .iter()
        .filter(|row| row.value.is_some())
        .map(|row| row.label.len())
        .max()
        .unwrap_or_default();
    let mut output = String::new();

    for row in rows {
        output.push_str(&row.render(label_width));
    }

    output
}

pub fn section(output: &mut String, title: &str, rows: &[StatusRow]) {
    writeln!(output, "{title}").unwrap();
    output.push_str(&render_rows(rows));
}

pub fn next_step(text: impl AsRef<str>) -> String {
    format!("  -> {}", text.as_ref())
}

pub fn content_block(output: &mut String, title: &str, contents: &str) {
    writeln!(output, "{title}").unwrap();
    writeln!(output, "{}", "-".repeat(title.len())).unwrap();
    write!(output, "{contents}").unwrap();
    if !contents.ends_with('\n') {
        writeln!(output).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_every_status_badge() {
        assert_eq!(Status::Ok.badge(), "[ok]");
        assert_eq!(Status::Warn.badge(), "[warn]");
        assert_eq!(Status::Miss.badge(), "[miss]");
        assert_eq!(Status::Error.badge(), "[error]");
        assert_eq!(Status::Run.badge(), "[run]");
        assert_eq!(Status::Pass.badge(), "[pass]");
        assert_eq!(Status::Fail.badge(), "[fail]");
        assert_eq!(Status::Skip.badge(), "[skip]");
        assert_eq!(Status::Plan.badge(), "[plan]");
        assert_eq!(Status::Write.badge(), "[write]");
        assert_eq!(Status::Keep.badge(), "[keep]");
        assert_eq!(Status::Refuse.badge(), "[refuse]");
    }

    #[test]
    fn renders_rows_with_and_without_values() {
        let rows = [
            row(Status::Ok, "Git repository"),
            row(Status::Ok, "Branch").value("main"),
        ];

        assert_eq!(
            render_rows(&rows),
            "  [ok]     Git repository\n  [ok]     Branch  main\n"
        );
    }

    #[test]
    fn aligns_values_per_section() {
        let rows = [
            row(Status::Ok, "Branch").value("main"),
            row(Status::Warn, "Git user.email").value("not configured"),
        ];

        assert_eq!(
            render_rows(&rows),
            "  [ok]     Branch          main\n  [warn]   Git user.email  not configured\n"
        );
    }

    #[test]
    fn indents_multiline_details() {
        let rows = [row(Status::Plan, "hook").detail("#!/bin/sh\nkoba run pre-commit")];

        assert_eq!(
            render_rows(&rows),
            "  [plan]   hook\n          #!/bin/sh\n          koba run pre-commit\n"
        );
    }

    #[test]
    fn renders_ascii_next_steps() {
        assert_eq!(next_step("Run `koba init`"), "  -> Run `koba init`");
    }
}
