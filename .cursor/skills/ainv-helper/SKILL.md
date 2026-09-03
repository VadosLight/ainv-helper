---
name: ainv-helper
description: >-
  Develop and maintain the ainv-helper macOS menu-bar utility (Rust, tray-icon/muda/tao).
  Use when working in the ainv-helper repo, building/installing the .app, fixing Dock/menu-bar
  behavior, first-launch admin privileges, single-instance lock, config paths, or README/architecture docs.
---

# AInv Helper — core

## What this is

Menu-bar only macOS app (no main window, no Dock icon). Stack: `tray-icon` + `muda` + `tao`, TOML config, `flexi_logger`, `fs2` lock.

## Layout

```
src/main.rs          # thin bootstrap
src/lib.rs           # module tree
src/app/             # UI: event loop, menu, icons
src/platform/        # macOS: autostart, privileges, instance, notify
src/{actions,android,hosts,config,logging}.rs
config/default.toml  # embedded defaults + migration source
scripts/build-app.sh
```

Do **not** flatten back to all modules in `main.rs`. Keep `lib` + `bin` split. Group only cohesive multi-file modules (`app/`, `platform/`); leave single-file domain modules flat.

## Build / run (critical)

macOS keeps the old process in the menu bar. After code changes always:

```bash
./scripts/build-app.sh restart
```

- `build` — release + `.app` under `target/release/`
- `install` — copy to `/Applications/AInv Helper.app`, update LaunchAgent
- `stop` — kill `ainv-helper`
- Plain `cargo build` does **not** replace the running menu-bar app

## Invariants from product history

1. **Menu bar only** — `LSUIElement` in `resources/Info.plist` + `ActivationPolicy::Accessory` in `app`
2. **Admin on first launch** — `platform::privileges::ensure_on_first_launch`; no standalone «Grant Admin» menu item
3. **Single instance** — `platform::instance::InstanceGuard` (`flock`); released on crash/exit
4. **Config path** — `~/Library/Application Support/ainv-helper/config.toml` (`dirs::config_dir` on macOS ≠ `~/.config`)
5. **Docs** — Russian `///` on public/private functions; keep README architecture in sync with `src/`
6. **No dead menu items** — remove actions that only re-request privileges or leftover status/CPU/battery features

## Config migration

Bump `CONFIG_VERSION` in `config.rs` and `config_version` in `config/default.toml` together. `migrate()` replaces `actions`/`hosts` from defaults when version is stale — required when menu items change, otherwise users keep old menus from Application Support.

## Verify

```bash
cargo test && cargo check
```

Then `./scripts/build-app.sh restart` for a live check.
