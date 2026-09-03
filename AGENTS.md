# AInv Helper — agent notes

## Skills (read before related work)

| Skill | When |
|-------|------|
| [ainv-helper](.cursor/skills/ainv-helper/SKILL.md) | Build/restart, layout, config path, Dock/menu-bar, single instance, docs |
| [ainv-helper-menu-toggles](.cursor/skills/ainv-helper-menu-toggles/SKILL.md) | New/changed menu toggles, tray indicator, AMIOProxy parity, config_version |
| [ainv-helper-hosts-android](.cursor/skills/ainv-helper-hosts-android/SKILL.md) | `/etc/hosts` + DNS flush, android `adb` proxy, amio-proxy status mismatches |

## Hard rules from product history

- After code changes: `./scripts/build-app.sh restart` (not only `cargo build`)
- Config lives in `~/Library/Application Support/ainv-helper/` — bump `config_version` when default menu changes
- Tray green = any of ios-sim-route **or** android-sim-proxy active
- Hosts write must flush mDNSResponder cache
- Prefer Type in TS only if touching other repos; here: Rust + Russian `///` docs
