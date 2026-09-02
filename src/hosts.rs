//! Чтение и запись /etc/hosts через привилегированные команды.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::config::{HostsConfig, HostsEntry};
use crate::privileges;

pub const HOSTS_PATH: &str = "/etc/hosts";
const BLOCK_BEGIN: &str = "# --- ainv-helper begin ---";
const BLOCK_END: &str = "# --- ainv-helper end ---";

pub const IOS_SIM_BEGIN: &str = "# BEGIN AMIO IOS SIMULATOR";
pub const IOS_SIM_END: &str = "# END AMIO IOS SIMULATOR";
pub const IOS_SIM_ENTRY: &str = "127.0.0.1 invest-test.alfabank.ru";

pub fn read() -> Result<String> {
    fs::read_to_string(HOSTS_PATH).with_context(|| format!("read {HOSTS_PATH}"))
}

pub fn apply(config: &HostsConfig) -> Result<()> {
    if !config.enabled {
        bail!("hosts management is disabled in config");
    }

    let current = read().unwrap_or_default();
    let updated = merge(&current, &config.entries);
    write(&updated)
}

pub fn clear_managed() -> Result<()> {
    let current = read().unwrap_or_default();
    let updated = remove_managed_block(&current);
    write(&updated)
}

pub fn is_ios_sim_route_active() -> bool {
    read()
        .map(|content| is_ios_sim_block_present(&content))
        .unwrap_or(false)
}

pub fn toggle_ios_sim_route() -> Result<bool> {
    let current = read().unwrap_or_default();
    let active = is_ios_sim_block_present(&current);
    let updated = if active {
        remove_ios_sim_block(&current)
    } else {
        append_ios_sim_block(&current)
    };
    write(&updated)?;
    Ok(!active)
}

pub fn ios_sim_block() -> String {
    format!("{IOS_SIM_BEGIN}\n{IOS_SIM_ENTRY}\n{IOS_SIM_END}\n")
}

fn is_ios_sim_block_present(content: &str) -> bool {
    content.contains(IOS_SIM_BEGIN)
        && content.contains(IOS_SIM_END)
        && content.contains(IOS_SIM_ENTRY)
}

fn append_ios_sim_block(content: &str) -> String {
    let mut result = content.trim_end().to_string();
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(&ios_sim_block());
    result
}

fn remove_ios_sim_block(content: &str) -> String {
    let Some(start) = content.find(IOS_SIM_BEGIN) else {
        return content.to_string();
    };

    let tail = &content[start..];
    let end_offset = tail
        .find(IOS_SIM_END)
        .map(|idx| idx + IOS_SIM_END.len())
        .unwrap_or(tail.len());

    let mut result = String::new();
    result.push_str(content[..start].trim_end());
    if end_offset < tail.len() {
        let rest = tail[end_offset..].trim_start_matches('\n');
        if !result.is_empty() && !rest.is_empty() {
            result.push('\n');
        }
        result.push_str(rest);
    }

    if !result.ends_with('\n') && !result.is_empty() {
        result.push('\n');
    }
    result
}

pub fn write(content: &str) -> Result<()> {
    let temp = staging_path();
    if let Some(parent) = temp.parent() {
        fs::create_dir_all(parent).context("create hosts staging directory")?;
    }
    fs::write(&temp, content).context("write staged hosts file")?;

    let cmd = format!(
        "cp {HOSTS_PATH} {HOSTS_PATH}.ainv-backup.$(date +%s) 2>/dev/null; cp {} {HOSTS_PATH}",
        shell_escape(&temp)
    );

    privileges::run_as_admin(&cmd).context("apply staged hosts file as root")?;
    log::info!("Updated {HOSTS_PATH}");
    Ok(())
}

fn merge(existing: &str, entries: &[HostsEntry]) -> String {
    let base = remove_managed_block(existing);
    let block = format_managed_block(entries);

    if block.is_empty() {
        return base.trim_end().to_string() + "\n";
    }

    let mut result = base.trim_end().to_string();
    if !result.is_empty() {
        result.push('\n');
    }
    result.push_str(&block);
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn remove_managed_block(content: &str) -> String {
    let Some(start) = content.find(BLOCK_BEGIN) else {
        return content.to_string();
    };

    let tail = &content[start..];
    let end_offset = tail
        .find(BLOCK_END)
        .map(|idx| idx + BLOCK_END.len())
        .unwrap_or(tail.len());

    let mut result = String::new();
    result.push_str(content[..start].trim_end());
    if end_offset < tail.len() {
        let rest = tail[end_offset..].trim_start_matches('\n');
        if !result.is_empty() && !rest.is_empty() {
            result.push('\n');
        }
        result.push_str(rest);
    }

    result
}

fn format_managed_block(entries: &[HostsEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let mut lines = vec![BLOCK_BEGIN.to_string()];
    for entry in entries {
        lines.push(entry.to_line());
    }
    lines.push(BLOCK_END.to_string());
    lines.join("\n")
}

fn staging_path() -> PathBuf {
    crate::config::config_dir().join("hosts.staging")
}

fn shell_escape(path: &PathBuf) -> String {
    let value = path.display().to_string();
    if value.contains(' ') || value.contains('\'') {
        format!("'{}'", value.replace('\'', "'\\''"))
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_and_replaces_managed_block() {
        let input = "127.0.0.1 localhost\n# --- ainv-helper begin ---\n127.0.0.1 old.test\n# --- ainv-helper end ---\n";
        let entries = vec![HostsEntry {
            ip: "127.0.0.1".into(),
            hostname: "new.test".into(),
        }];
        let merged = merge(input, &entries);
        assert!(merged.contains("new.test"));
        assert!(!merged.contains("old.test"));
        assert!(merged.contains("127.0.0.1 localhost"));
    }

    #[test]
    fn detects_ios_sim_block() {
        let content = format!("127.0.0.1 localhost\n{}", ios_sim_block());
        assert!(is_ios_sim_block_present(&content));
    }

    #[test]
    fn toggles_ios_sim_block() {
        let base = "127.0.0.1 localhost\n";
        let with_block = append_ios_sim_block(base);
        assert!(is_ios_sim_block_present(&with_block));
        let cleared = remove_ios_sim_block(&with_block);
        assert!(!is_ios_sim_block_present(&cleared));
    }
}

