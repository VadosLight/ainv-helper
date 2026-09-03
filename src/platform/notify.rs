//! Системные уведомления macOS через `osascript`.

use std::process::Command;

use anyhow::{Context, Result};

/// Показывает модальный диалог с ошибкой (title + текст причины).
pub fn show_error(title: &str, message: &str) -> Result<()> {
    let script = format!(
        "display dialog {} with title {} buttons {{\"OK\"}} default button 1 with icon stop",
        applescript_quote(message),
        applescript_quote(title),
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("show error dialog via osascript")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("Error dialog dismissed or failed: {stderr}");
    }

    Ok(())
}

/// Экранирует строку для вставки в AppleScript.
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::applescript_quote;

    #[test]
    fn escapes_quotes() {
        assert_eq!(applescript_quote(r#"adb "missing""#), r#""adb \"missing\"""#);
    }
}
