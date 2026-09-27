use std::io::IsTerminal;
use std::sync::OnceLock;

fn color() -> bool {
    static COLOR: OnceLock<bool> = OnceLock::new();
    *COLOR.get_or_init(|| std::io::stdout().is_terminal())
}

fn paint(code: &str, text: &str) -> String {
    if color() {
        format!("\x1b[{code}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

pub fn bold(text: &str) -> String {
    paint("1", text)
}

pub fn dim(text: &str) -> String {
    paint("2", text)
}

/// Marks a value red with ⚠ when it crosses a threshold.
pub fn flag(text: &str, warn: bool) -> String {
    if warn {
        paint("31", &format!("{text} ⚠"))
    } else {
        text.to_string()
    }
}
