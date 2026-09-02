//! Логирование в файл и stderr (без UI-уведомлений об ошибках).

use std::path::PathBuf;

use anyhow::{Context, Result};
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};

use crate::config;

/// Инициализирует ротируемый file-logger в каталоге конфигурации.
pub fn init() -> Result<()> {
    let log_dir = config_dir();
    std::fs::create_dir_all(&log_dir).context("create log directory")?;

    Logger::try_with_env_or_str("info")
        .context("configure logger")?
        .log_to_file(
            FileSpec::default()
                .directory(&log_dir)
                .basename("ainv-helper")
                .suffix("log"),
        )
        .rotate(
            Criterion::Size(512_000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(3),
        )
        .duplicate_to_stderr(Duplicate::Info)
        .start()
        .context("start logger")?;

    log::info!("Logging initialized at {}", log_dir.display());
    Ok(())
}

/// Возвращает каталог для log-файлов (совпадает с config dir).
fn config_dir() -> PathBuf {
    config::config_dir()
}
