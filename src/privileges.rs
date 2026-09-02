//! Запрос прав администратора через нативный диалог macOS.
//!
//! На первом запуске показывает системный prompt (osascript) и сохраняет
//! флаг согласия. Дальнейшие изменения /etc/hosts выполняются через
//! `do shell script … with administrator privileges`.

use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;

const ADMIN_STATE_FILE: &str = ".admin_granted";

/// Путь к файлу-флагу «права администратора получены».
pub fn state_path() -> std::path::PathBuf {
    config::config_dir().join(ADMIN_STATE_FILE)
}

/// Проверяет, был ли уже успешный запрос прав администратора.
pub fn is_granted() -> bool {
    state_path().exists()
}

/// Запрашивает права администратора при первом запуске. Без согласия приложение не стартует.
pub fn ensure_on_first_launch() -> Result<()> {
    if is_granted() {
        log::debug!("Admin privileges already granted on first launch");
        return Ok(());
    }

    log::info!("First launch — requesting administrator privileges for /etc/hosts");

    explain_admin_need()?;
    verify_admin_access()?;
    mark_granted()?;

    log::info!("Administrator privileges granted");
    Ok(())
}

/// Повторный запрос прав администратора (пункт меню).
pub fn request_again() -> Result<()> {
    explain_admin_need()?;
    verify_admin_access()?;
    mark_granted()?;
    log::info!("Administrator privileges re-granted");
    Ok(())
}

/// Выполняет shell-команду с правами root через `osascript`.
pub fn run_as_admin(shell_command: &str) -> Result<()> {
    let script = format!(
        "do shell script {} with administrator privileges",
        applescript_quote(shell_command)
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("run osascript with administrator privileges")?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("User canceled") || stderr.contains("-128") {
        bail!("administrator access denied by user");
    }

    bail!("privileged command failed: {stderr}");
}

/// Показывает диалог с объяснением, зачем нужны права администратора.
fn explain_admin_need() -> Result<()> {
    let script = r#"display dialog "AInv Helper требует права администратора для изменения файла /etc/hosts.

Нажмите «Продолжить», затем введите пароль в системном диалоге." buttons {"Отмена", "Продолжить"} default button 2 with title "AInv Helper" with icon caution"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("show admin explanation dialog")?;

    if !output.status.success() {
        bail!("administrator access setup cancelled");
    }

    Ok(())
}

/// Проверяет доступ к `/etc/hosts` через привилегированную команду.
fn verify_admin_access() -> Result<()> {
    run_as_admin("test -f /etc/hosts && test -r /etc/hosts")
}

/// Записывает флаг успешного получения прав администратора.
fn mark_granted() -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create config directory for admin state")?;
    }
    fs::write(&path, "granted\n").context("write admin state file")?;
    Ok(())
}

/// Экранирует строку для вставки в AppleScript.
fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::applescript_quote;

    /// Проверяет экранирование кавычек в AppleScript-строках.
    #[test]
    fn escapes_quotes() {
        assert_eq!(applescript_quote(r#"say "hi""#), r#""say \"hi\"""#);
    }
}
