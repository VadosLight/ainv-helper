//! Порядок старта: логирование → конфиг → права админа → автозапуск → UI.

mod actions;
mod app;
mod autostart;
mod config;
mod hosts;
mod icons;
mod instance;
mod logging;
mod privileges;

use anyhow::Result;

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


