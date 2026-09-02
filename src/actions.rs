//! Выполнение быстрых действий из конфига (shell-команды, hosts).

use std::process::Command;

use anyhow::{Context, Result};

use crate::config::{ActionConfig, ActionType, Config};
use crate::hosts;

pub fn execute(action: &ActionConfig, config: &Config) -> Result<()> {
    log::info!("Executing action: {}", action.label);

    match action.action_type {
        ActionType::Shell => run_shell(&action.command),
        ActionType::HostsApply => hosts::apply(&config.hosts),
        ActionType::HostsClear => hosts::clear_managed(),
        ActionType::Header => Ok(()),
        ActionType::IosSimRoute => {
            let enabled = hosts::toggle_ios_sim_route()?;
            log::info!(
                "ios-sim-route {}",
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(())
        }
    }
}

fn run_shell(command: &str) -> Result<()> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .with_context(|| format!("run shell command: {command}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!(
            "Command exited with {}: {stderr}",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(())
}



