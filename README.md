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
```

| Поле | Описание |
|------|----------|
| `poll_interval_secs` | Интервал обновления tray-индикатора (сек) |
| `actions` | Пункты меню; `action_type`: `header`, `ios_sim_route`, `shell`, `hosts_apply`, `hosts_clear` |

Повторный запрос прав: **Grant Administrator Access…** в меню.

Редактирование: **Edit Configuration…** в меню. После изменений — **Reload Configuration**.

Логи: `~/Library/Application Support/ainv-helper/ainv-helper.log`

## Архитектура

```
main.rs       → точка входа: логирование, конфиг, права админа, автозапуск, UI
app.rs        → event loop (tao), NSStatusItem + NSMenu (tray-icon/muda)
config.rs     → загрузка TOML, пути к конфигу и .app bundle
privileges.rs → запрос прав администратора (osascript), первый запуск
hosts.rs      → чтение/запись /etc/hosts через privileged shell
autostart.rs  → LaunchAgent plist + launchctl
icons.rs      → RGBA-иконки tray «AINV» и пунктов меню
actions.rs    → выполнение действий из конфига
logging.rs    → flexi_logger в файл + stderr
```

Поток работы:

1. `main` инициализирует подсистемы и передаёт конфиг в `App::run`.
2. `App` создаёт иконку в строке меню и нативное меню.
3. Event loop обрабатывает клики по меню и периодически обновляет иконку.
4. Ошибки пишутся в лог, UI-уведомления не показываются.

## Структура проекта

```
src/                  исходный код
config/default.toml   конфиг по умолчанию (встраивается в бинарник)
resources/Info.plist  LSUIElement + ActivationPolicy::Accessory — без иконки в Dock
scripts/build-app.sh  сборка .app bundle
```
