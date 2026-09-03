//! Порядок старта: логирование → конфиг → права админа → автозапуск → UI.

use anyhow::Result;

use ainv_helper::app;
use ainv_helper::config;
use ainv_helper::logging;
use ainv_helper::platform::{autostart, instance, privileges};

/// Точка входа: инициализирует подсистемы и запускает menu bar UI.
fn main() -> Result<()> {
    logging::init()?;
    let _instance = instance::InstanceGuard::acquire()?;
    log::info!(
        "Starting AInv Helper v{} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_NAME")
    );

    let cfg = config::load_or_create()?;
    privileges::ensure_on_first_launch()?;
    autostart::register_on_first_launch()?;

    app::App::run(cfg)
}
