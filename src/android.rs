//! Управление Android emulator system HTTP proxy через `adb`.
//!
//! Реализует ту же логику, что AMIOProxy `emulator-proxy-on` / `emulator-proxy-off`:
//! `settings put/delete global http_proxy` (+ host/port).

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Result, bail};

/// Host, который эмулятор видит как хост-машину Mac (по умолчанию как в AMIOProxy).
const DEFAULT_ANDROID_PROXY_HOST: &str = "10.0.2.2";
/// Порт локального AMIO proxy.
const DEFAULT_LISTEN_PORT: u16 = 9140;

/// Путь к `adb` (кэшируется после первого поиска).
fn adb_bin() -> &'static str {
    static ADB: OnceLock<String> = OnceLock::new();
    ADB.get_or_init(find_adb).as_str()
}

/// Host и порт proxy для эмулятора (env перекрывает дефолты AMIOProxy).
pub fn proxy_endpoint() -> (String, u16) {
    let host = env::var("AIO_PROXY_ANDROID_HOST")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_ANDROID_PROXY_HOST.to_string());
    let port = env::var("AIO_PROXY_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_LISTEN_PORT);
    (host, port)
}

/// Проверяет, включён ли Android system HTTP proxy на подключённом устройстве.
pub fn is_proxy_active() -> bool {
    match read_setting("http_proxy") {
        Ok(value) => is_proxy_value_active(&value),
        Err(err) => {
            log::debug!("android proxy status unavailable: {err:#}");
            false
        }
    }
}

/// Переключает Android system proxy; возвращает новое состояние (`true` = включён).
pub fn toggle_proxy() -> Result<bool> {
    if is_proxy_active() {
        clear_proxy()?;
        Ok(false)
    } else {
        let (host, port) = proxy_endpoint();
        set_proxy(&host, port)?;
        Ok(true)
    }
}

/// Включает system proxy на `host:port` (аналог `emulator-proxy-on`).
pub fn set_proxy(host: &str, port: u16) -> Result<()> {
    ensure_device()?;
    run_adb(&[
        "shell",
        "settings",
        "put",
        "global",
        "http_proxy",
        &format!("{host}:{port}"),
    ])?;
    run_adb(&[
        "shell",
        "settings",
        "put",
        "global",
        "global_http_proxy_host",
        host,
    ])?;
    run_adb(&[
        "shell",
        "settings",
        "put",
        "global",
        "global_http_proxy_port",
        &port.to_string(),
    ])?;
    log::info!("Android system proxy set: {host}:{port}");
    Ok(())
}

/// Выключает system proxy (аналог `emulator-proxy-off`).
pub fn clear_proxy() -> Result<()> {
    ensure_device()?;
    let _ = run_adb_allow_failure(&["shell", "settings", "delete", "global", "http_proxy"]);
    let _ = run_adb_allow_failure(&[
        "shell",
        "settings",
        "delete",
        "global",
        "global_http_proxy_host",
    ]);
    let _ = run_adb_allow_failure(&[
        "shell",
        "settings",
        "delete",
        "global",
        "global_http_proxy_port",
    ]);
    log::info!("Android system proxy cleared");
    Ok(())
}

/// Считает proxy активным, если значение не пустое / null / :0.
fn is_proxy_value_active(value: &str) -> bool {
    let trimmed = value.trim();
    !(trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("null")
        || trimmed == ":0"
        || trimmed == "null:null")
}

/// Читает global setting с устройства.
fn read_setting(name: &str) -> Result<String> {
    let output = run_adb_allow_failure(&["shell", "settings", "get", "global", name])?;
    Ok(output.trim().to_string())
}

/// Убеждается, что adb видит хотя бы одно устройство в состоянии `device`.
fn ensure_device() -> Result<()> {
    let serials = connected_device_serials()?;
    if serials.is_empty() {
        bail!("no Android emulator/device connected (adb devices)");
    }
    Ok(())
}

/// Список serial устройств в состоянии `device`.
fn connected_device_serials() -> Result<Vec<String>> {
    let output = run_raw_adb(&["devices"])?;
    let serials = output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let serial = parts.next()?;
            let state = parts.next()?;
            (state == "device").then(|| serial.to_string())
        })
        .collect();
    Ok(serials)
}

/// Выбирает serial: env → emulator-* → первое устройство.
fn selected_device_serial() -> Result<Option<String>> {
    if let Ok(serial) = env::var("AIO_PROXY_ADB_SERIAL").or_else(|_| env::var("ANDROID_SERIAL")) {
        if !serial.is_empty() {
            return Ok(Some(serial));
        }
    }

    let serials = connected_device_serials()?;
    if serials.is_empty() {
        return Ok(None);
    }

    Ok(Some(
        serials
            .iter()
            .find(|s| s.starts_with("emulator-"))
            .cloned()
            .unwrap_or_else(|| serials[0].clone()),
    ))
}

/// Запускает adb с `-s <serial>` при необходимости; падает при ненулевом exit.
fn run_adb(args: &[&str]) -> Result<String> {
    run_adb_inner(args, false)
}

/// Запускает adb, допускает ненулевой exit code.
fn run_adb_allow_failure(args: &[&str]) -> Result<String> {
    run_adb_inner(args, true)
}

fn run_adb_inner(args: &[&str], allow_failure: bool) -> Result<String> {
    let mut full_args: Vec<String> = Vec::new();
    if args.first().copied() != Some("devices") && !args.iter().any(|a| *a == "-s") {
        if let Some(serial) = selected_device_serial()? {
            full_args.push("-s".into());
            full_args.push(serial);
        }
    }
    full_args.extend(args.iter().map(|s| (*s).to_string()));

    let output = Command::new(adb_bin())
        .args(&full_args)
        .output()
        .map_err(|err| map_adb_spawn_error(err, &full_args))?;

    if !output.status.success() && !allow_failure {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "adb {} failed: {}",
            full_args.join(" "),
            if !stderr.trim().is_empty() {
                stderr
            } else {
                stdout
            }
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Запускает adb без выбора устройства (для `devices`).
fn run_raw_adb(args: &[&str]) -> Result<String> {
    let output = Command::new(adb_bin())
        .args(args)
        .output()
        .map_err(|err| map_adb_spawn_error(err, &args.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn map_adb_spawn_error(err: std::io::Error, args: &[String]) -> anyhow::Error {
    if err.kind() == std::io::ErrorKind::NotFound {
        anyhow::anyhow!("adb not found — install android-platform-tools or set AIO_PROXY_ADB")
    } else {
        anyhow::anyhow!("run adb {}: {err}", args.join(" "))
    }
}

/// Ищет `adb` в env / Android SDK / PATH (как AMIOProxy `findAdb`).
fn find_adb() -> String {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(explicit) = env::var("AIO_PROXY_ADB") {
        if !explicit.is_empty() {
            candidates.push(PathBuf::from(explicit));
        }
    }
    if let Ok(home) = env::var("ANDROID_HOME") {
        candidates.push(Path::new(&home).join("platform-tools/adb"));
    }
    if let Ok(root) = env::var("ANDROID_SDK_ROOT") {
        candidates.push(Path::new(&root).join("platform-tools/adb"));
    }
    if let Some(home) = dirs::home_dir() {
        candidates.push(home.join("Library/Android/sdk/platform-tools/adb"));
    }
    candidates.push(PathBuf::from("adb"));

    candidates
        .into_iter()
        .find(|path| path.as_os_str() == "adb" || path.exists())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "adb".to_string())
}

#[cfg(test)]
mod tests {
    use super::is_proxy_value_active;

    #[test]
    fn detects_active_proxy_values() {
        assert!(is_proxy_value_active("10.0.2.2:9140"));
        assert!(!is_proxy_value_active("null"));
        assert!(!is_proxy_value_active(":0"));
        assert!(!is_proxy_value_active(""));
    }
}
