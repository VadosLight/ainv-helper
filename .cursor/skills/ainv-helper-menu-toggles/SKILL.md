---
name: ainv-helper-menu-toggles
description: >-
  Add or change ainv-helper menu toggles (ios_sim_route, android_sim_proxy, headers, tray indicator).
  Use when adding ActionType, IconMenuItem green/red checks, AINV tray dot color, config migration for
  new menu items, or mirroring terminal AMIOProxy on/off/status commands.
---

# AInv Helper — menu toggles

## Reference for AMIO parity

Terminal: `scripts/dev/AMIOProxy/` (`index.ts`, `iosHosts.ts`, `android.ts`, `config.ts`).

| Menu label | `action_type` | AMIOProxy commands | Implementation |
|---|---|---|---|
| `toggle ios-sim-route` | `ios_sim_route` | `ios-simulator-route-on/off` | `hosts.rs` block markers |
| `toggle-android-sim-proxy` | `android_sim_proxy` | `emulator-proxy-on/off` | `android.rs` via `adb` |

Defaults for Android (same as AMIO): host `10.0.2.2`, port `9140`; override via `AIO_PROXY_ANDROID_HOST` / `AIO_PROXY_PORT` / `AIO_PROXY_ADB`.

## Checklist for a new toggle

1. Add `ActionType` variant (`serde rename_all = "snake_case"`) in `config.rs`
2. Handle in `actions::execute`
3. In `app/mod.rs` `build_menu`: `IconMenuItem` with green ✓ / red ✗
4. Add `sync_*_item` + poll + rebuild reset (`last_* = None`)
5. On success **and** after state change: call `refresh_tray_indicator()`
6. On failure (at least for Android): `platform::notify::show_error` with `{err:#}`
7. Add `[[actions]]` to `config/default.toml`
8. Bump `CONFIG_VERSION` (and default `config_version`) so existing installs migrate
9. Update README action_type list

## Tray indicator rule

Green AINV dot if **any** of:
- `hosts::any_hosts_route_active(config)` (ios sim block present)
- `android::is_proxy_active()`

Yellow only when all checked toggles are off. Menu checkmarks alone are not enough — always refresh tray after toggle.

Do **not** put a useless standalone «Grant Administrator Access» item; privileges are requested on first launch and when writing hosts.

## Headers

`action_type = "header"` → disabled `MenuItem` (section label, no action).

## iOS hosts block (exact)

```
# BEGIN AMIO IOS SIMULATOR
127.0.0.1 invest-test.alfabank.ru
# END AMIO IOS SIMULATOR
```

Active = all three lines present. After any `/etc/hosts` write, flush DNS (see skill `ainv-helper-hosts-android`).
