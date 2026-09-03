---
name: ainv-helper-hosts-android
description: >-
  Fix ainv-helper /etc/hosts writes, DNS cache flush, and Android emulator HTTP proxy via adb.
  Use when ios-sim-route status shows hosts enabled but DNS is not 127.0.0.1, when debugging
  amio-proxy ios-simulator-route-status, or when android-sim-proxy / adb settings fail.
---

# AInv Helper — hosts & Android proxy

## Hosts write must flush DNS

AMIOProxy installs hosts then runs:

```bash
/usr/bin/dscacheutil -flushcache
/usr/bin/killall -HUP mDNSResponder
```

`hosts::write` in ainv-helper must do the same inside the privileged `osascript` shell (via `install` + flush). Without flush, `dscacheutil -q host` / apps keep the corporate IP (e.g. `10.210.5.126`) even though `/etc/hosts` already has `127.0.0.1 invest-test.alfabank.ru`.

### Diagnose

```bash
grep -n 'AMIO IOS\|invest-test' /etc/hosts
dscacheutil -q host -a name invest-test.alfabank.ru
# dig ignores /etc/hosts — do not use dig as proof of hosts routing
pnpm --filter=@terminal/pwa run amio-proxy ios-simulator-route-status
```

Warning «hosts содержит маршрут, но DNS еще не возвращает 127.0.0.1» → flush cache (or re-toggle route after ainv-helper fix).

Manual flush:

```bash
sudo dscacheutil -flushcache
sudo killall -HUP mDNSResponder
```

## Android system proxy

Mirror `setAndroidProxy` / `clearAndroidProxy` from terminal `AMIOProxy/android.ts`:

- on: `settings put global http_proxy`, `global_http_proxy_host`, `global_http_proxy_port`
- off: `settings delete` those keys (`allowFailure`)
- active: `http_proxy` not empty / not `null` / not `:0`

Require a connected `adb` device (`emulator-*` preferred). Failures → `platform::notify::show_error` (system dialog with reason), still log with `log::error!`.

## Privileges

Hosts edits go through `platform::privileges::run_as_admin`. First launch: `ensure_on_first_launch`. Do not add a menu-only re-grant action.
