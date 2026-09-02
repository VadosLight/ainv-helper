//! Опрос системного статуса для индикатора в строке меню.
//!
//! `battery` — pmset, `cpu` — sysinfo. Возвращает числовое значение и подпись.

use std::process::Command;

use anyhow::{Context, Result};
use sysinfo::System;

use crate::config::StatusSource;
use crate::icons;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSnapshot {
    pub value: u8,
    pub label: String,
}

pub fn poll(source: StatusSource) -> Result<StatusSnapshot> {
    match source {
        StatusSource::Battery => poll_battery(),
        StatusSource::Cpu => poll_cpu(),
    }
}

fn poll_battery() -> Result<StatusSnapshot> {
    let output = Command::new("pmset")
        .args(["-g", "batt"])
        .output()
        .context("run pmset")?;

    let text = String::from_utf8_lossy(&output.stdout);

    let percent = text
        .split_whitespace()
        .find_map(|token| {
            token
                .trim_end_matches(';')
                .strip_suffix('%')
                .and_then(|n| n.parse::<u8>().ok())
        })
        .unwrap_or(100);

    let charging = text.contains("AC Power") || text.contains("charging");

    let label = if charging {
        format!("{percent}%⚡")
    } else {
        format!("{percent}%")
    };

    Ok(StatusSnapshot {
        value: percent,
        label,
    })
}

fn poll_cpu() -> Result<StatusSnapshot> {
    let mut system = System::new();
    system.refresh_cpu_usage();
    std::thread::sleep(std::time::Duration::from_millis(200));
    system.refresh_cpu_usage();

    let usage = system.global_cpu_usage().round() as u8;
    Ok(StatusSnapshot {
        value: usage,
        label: format!("{usage}%"),
    })
}

pub fn icon_for(snapshot: &StatusSnapshot, source: StatusSource) -> tray_icon::Icon {
    let color = match source {
        StatusSource::Battery => icons::battery_color(snapshot.value),
        StatusSource::Cpu => icons::cpu_color(snapshot.value),
    };

    icons::make_battery_icon(snapshot.value, color)
}

