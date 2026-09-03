//! Конфигурация приложения (TOML).
//!
//! Файл: `~/Library/Application Support/ainv-helper/config.toml` (macOS)
//! При отсутствии создаётся из `config/default.toml`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const DEFAULT_CONFIG: &str = include_str!("../config/default.toml");

/// Текущая версия схемы конфигурации; при несовпадении выполняется миграция.
pub const CONFIG_VERSION: u32 = 4;

/// Корневая структура конфигурации приложения.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_config_version")]
    pub config_version: u32,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub hosts: HostsConfig,
    #[serde(default)]
    pub actions: Vec<ActionConfig>,
}

/// Настройки управления `/etc/hosts` через конфиг.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub entries: Vec<HostsEntry>,
}

/// Одна запись hosts: IP + hostname.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostsEntry {
    pub ip: String,
    pub hostname: String,
}

impl HostsEntry {
    /// Форматирует запись в строку для `/etc/hosts`.
    pub fn to_line(&self) -> String {
        format!("{} {}", self.ip.trim(), self.hostname.trim())
    }
}

impl Default for HostsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            entries: Vec::new(),
        }
    }
}

/// Описание одного пункта меню из конфига.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfig {
    pub label: String,
    #[serde(default)]
    pub command: String,
    #[serde(default = "default_action_type")]
    pub action_type: ActionType,
}

/// Тип действия пункта меню.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Shell,
    HostsApply,
    HostsClear,
    Header,
    IosSimRoute,
    AndroidSimProxy,
}

/// Значение по умолчанию для интервала опроса (30 сек).
fn default_poll_interval() -> u64 {
    30
}

/// Версия конфига для старых файлов без поля `config_version`.
fn default_config_version() -> u32 {
    1
}

/// Тип действия по умолчанию.
fn default_action_type() -> ActionType {
    ActionType::Shell
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_CONFIG).expect("default config must be valid")
    }
}

/// Возвращает каталог конфигурации приложения.
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ainv-helper")
}

/// Полный путь к файлу `config.toml`.
pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Загружает конфиг или создаёт дефолтный; выполняет миграцию при необходимости.
pub fn load_or_create() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        fs::create_dir_all(path.parent().context("config parent dir")?)
            .context("create config directory")?;
        let config = Config::default();
        save(&config)?;
        log::info!("Created default config at {}", path.display());
        return Ok(config);
    }

    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut config: Config =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;

    if migrate(&mut config)? {
        save(&config)?;
        log::info!("Migrated config to version {}", CONFIG_VERSION);
    }

    Ok(config)
}

/// Сериализует и сохраняет конфиг на диск.
pub fn save(config: &Config) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("create config directory")?;
    }
    let raw = toml::to_string_pretty(config).context("serialize config")?;
    fs::write(&path, raw).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Обновляет устаревшие секции конфига до `CONFIG_VERSION`.
fn migrate(config: &mut Config) -> Result<bool> {
    if config.config_version >= CONFIG_VERSION {
        return Ok(false);
    }

    let default = Config::default();
    config.actions = default.actions;
    config.hosts = default.hosts;
    config.config_version = CONFIG_VERSION;
    Ok(true)
}

/// Открывает `config.toml` в TextEdit.
pub fn open_config_in_editor() -> Result<()> {
    let path = config_path();
    std::process::Command::new("open")
        .arg("-t")
        .arg(path)
        .spawn()
        .context("open config in TextEdit")?;
    Ok(())
}

/// Возвращает путь к `.app` bundle, если процесс запущен из него.
pub fn app_bundle_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| find_app_bundle(&exe))
}

/// Ищет ancestor с расширением `.app` начиная от пути к исполняемому файлу.
fn find_app_bundle(path: &Path) -> Option<PathBuf> {
    for ancestor in path.ancestors() {
        if ancestor.extension().and_then(|e| e.to_str()) == Some("app") {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}


