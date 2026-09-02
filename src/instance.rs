//! Гарантия единственного экземпляра приложения.

use std::fs::{File, OpenOptions};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use fs2::FileExt;

use crate::config;

/// Держит exclusive lock; при завершении процесса lock снимается автоматически.
pub struct InstanceGuard {
    _lock: File,
}

impl InstanceGuard {
    /// Захватывает lock-файл. Ошибка, если приложение уже запущено.
    pub fn acquire() -> Result<Self> {
        let path = lock_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).context("create lock directory")?;
        }

        let lock = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .with_context(|| format!("open lock file {}", path.display()))?;

        if lock.try_lock_exclusive().is_err() {
            bail!("AInv Helper уже запущен");
        }

        Ok(Self { _lock: lock })
    }
}

/// Путь к lock-файлу в каталоге конфигурации.
fn lock_path() -> PathBuf {
    config::config_dir().join("ainv-helper.lock")
}
