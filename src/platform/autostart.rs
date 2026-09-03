//! Автозапуск через LaunchAgent (~/Library/LaunchAgents).
//!
//! Требует запуск из .app bundle — путь к бинарнику берётся из Contents/MacOS.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};

const LAUNCH_AGENT_LABEL: &str = "com.ainv-helper";

/// Путь к plist-файлу LaunchAgent.
pub fn launch_agent_path() -> PathBuf {
    dirs::home_dir()
        .expect("home directory")
        .join("Library/LaunchAgents")
        .join(format!("{LAUNCH_AGENT_LABEL}.plist"))
}

/// Проверяет, зарегистрирован ли автозапуск (plist существует).
pub fn is_enabled() -> bool {
    launch_agent_path().exists()
}

/// Включает или отключает автозапуск через `launchctl load/unload`.
pub fn set_enabled(enabled: bool, app_bundle: Option<&PathBuf>) -> Result<()> {
    let plist_path = launch_agent_path();

    if enabled {
        let bundle = app_bundle
            .cloned()
            .or_else(crate::config::app_bundle_path)
            .context("autostart requires running from a .app bundle")?;

        let executable = bundle.join("Contents/MacOS/ainv-helper");

        if !executable.exists() {
            bail!(
                "executable not found at {} — build the .app bundle first",
                executable.display()
            );
        }

        let agents_dir = plist_path.parent().context("LaunchAgents parent")?;
        fs::create_dir_all(agents_dir).context("create LaunchAgents directory")?;

        let plist = build_plist(&executable);
        fs::write(&plist_path, plist).context("write LaunchAgent plist")?;

        let status = Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&plist_path)
            .status()
            .context("launchctl load")?;

        if !status.success() {
            log::warn!("launchctl load returned non-zero: {status}");
        }

        log::info!("Autostart enabled via LaunchAgent");
    } else if plist_path.exists() {
        let _ = Command::new("launchctl")
            .args(["unload", "-w"])
            .arg(&plist_path)
            .status();

        fs::remove_file(&plist_path).context("remove LaunchAgent plist")?;
        log::info!("Autostart disabled");
    }

    Ok(())
}

/// Формирует XML plist для LaunchAgent с указанным путём к бинарнику.
fn build_plist(executable: &PathBuf) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LAUNCH_AGENT_LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>ProcessType</key>
    <string>Interactive</string>
</dict>
</plist>
"#,
        executable.display()
    )
}

/// Регистрирует автозапуск при первом запуске из `.app`, если plist ещё нет.
pub fn register_on_first_launch() -> Result<()> {
    if is_enabled() {
        return Ok(());
    }

    if crate::config::app_bundle_path().is_some() {
        set_enabled(true, None)?;
    } else {
        log::info!("Skipping autostart registration — not running from .app bundle");
    }

    Ok(())
}
