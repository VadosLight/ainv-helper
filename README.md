# AInv Helper

Фоновое macOS-приложение для строки меню: нативное выпадающее меню, индикатор статуса и быстрые действия без основного окна.

## Требования

- macOS 12+
- [Rust](https://rustup.rs/) (stable)

## Сборка

```bash
# debug
cargo build

# release + .app bundle
./scripts/build-app.sh

# установка в /Applications и остановка старого процесса
./scripts/build-app.sh install

# собрать, установить и запустить
./scripts/build-app.sh restart
```

> **Важно:** macOS не перезапускает приложение, если оно уже висит в строке меню.  
> Перед обновлением: **Quit AInv Helper** в меню, или `./scripts/build-app.sh stop`.

Результат: `target/release/AInv Helper.app`

## Установка

```bash
./scripts/build-app.sh install
open "/Applications/AInv Helper.app"
```

Или одной командой (сборка + установка + запуск):

```bash
./scripts/build-app.sh restart
```

При первом запуске приложение запрашивает права администратора (нативный диалог macOS) для возможности изменять `/etc/hosts`. Без согласия приложение не запустится.

При первом запуске из `.app` также регистрируется автозапуск через LaunchAgent. Переключатель «Launch at Login» доступен в меню.

> Автозапуск работает только при запуске из `.app` bundle — отдельный бинарник macOS не поддерживает Login Item / LaunchAgent корректно.

## Настройка

Конфиг создаётся автоматически:

```
~/Library/Application Support/ainv-helper/config.toml
```

> На macOS `dirs::config_dir()` указывает на `Application Support`, не на `~/.config`.

Пример:

```toml
poll_interval_secs = 30

[[actions]]
label = "toggle ios-sim-route"
action_type = "ios_sim_route"

[[actions]]
label = "toggle-android-sim-proxy"
action_type = "android_sim_proxy"
```

| Поле | Описание |
|------|----------|
| `poll_interval_secs` | Интервал обновления tray-индикатора (сек) |
| `actions` | Пункты меню; `action_type`: `header`, `ios_sim_route`, `android_sim_proxy`, `shell`, `hosts_apply`, `hosts_clear` |

`android_sim_proxy` включает/выключает Android system HTTP proxy через `adb` (как AMIOProxy `emulator-proxy-on` / `emulator-proxy-off`). По умолчанию `10.0.2.2:9140`; перекрывается `AIO_PROXY_ANDROID_HOST` / `AIO_PROXY_PORT`. Нужен запущенный эмулятор и `adb` в PATH (или `AIO_PROXY_ADB`).

Редактирование: **Edit Configuration…** в меню. После изменений — **Reload Configuration**.

Логи: `~/Library/Application Support/ainv-helper/ainv-helper.log`

## Архитектура

```
main.rs            → bootstrap: логирование, конфиг, права, автозапуск, UI
lib.rs             → дерево модулей crate
app/               → event loop (tao), NSStatusItem + NSMenu (tray-icon/muda)
  icons.rs         → RGBA-иконки tray «AINV» и пунктов меню
platform/          → macOS: LaunchAgent, admin privileges, single instance, alerts
config.rs          → загрузка TOML, пути к конфигу и .app bundle
hosts.rs           → чтение/запись /etc/hosts через privileged shell
android.rs         → Android emulator system proxy через adb
actions.rs         → исполнение действий из конфига
logging.rs         → flexi_logger в файл + stderr
```

Поток работы:

1. `main` инициализирует подсистемы и передаёт конфиг в `App::run`.
2. `App` создаёт иконку в строке меню и нативное меню.
3. Event loop обрабатывает клики по меню и периодически обновляет иконку.
4. Ошибки пишутся в лог, UI-уведомления не показываются.

## Структура проекта

```
src/                  исходный код (lib + bin)
  app/                UI: event loop, меню, иконки
  platform/           macOS integrations
config/default.toml   конфиг по умолчанию (встраивается в бинарник)
resources/Info.plist  LSUIElement + ActivationPolicy::Accessory — без иконки в Dock
scripts/build-app.sh  сборка .app bundle
.cursor/skills/       Agent Skills для работы с репозиторием
AGENTS.md             указатель на skills и жёсткие правила
```
