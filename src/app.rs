//! Ядро приложения: event loop, NSStatusItem и NSMenu.
//!
//! `tray-icon`/`muda` оборачивают AppKit (NSStatusItem, NSMenu).
//! Цикл tao опрашивает MenuEvent и периодически обновляет иконку статуса.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use muda::{CheckMenuItem, IconMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::run_return::EventLoopExtRunReturn;
use tray_icon::{TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::actions;
use crate::autostart;
use crate::config::{self, ActionType, Config};
use crate::hosts;
use crate::icons;
use crate::privileges;

const MENU_QUIT: &str = "quit";
const MENU_AUTOSTART: &str = "autostart";
const MENU_EDIT_CONFIG: &str = "edit_config";
const MENU_RELOAD_CONFIG: &str = "reload_config";
const MENU_GRANT_ADMIN: &str = "grant_admin";

/// Состояние menu bar приложения: tray icon, меню и кэш индикаторов.
pub struct App {
    config: Config,
    tray: TrayIcon,
    menu: Menu,
    menu_ids: MenuIds,
    last_poll: Instant,
    last_tray_hosts_active: Option<bool>,
    last_ios_sim_active: Option<bool>,
}

/// Идентификаторы системных и пользовательских пунктов меню.
struct MenuIds {
    quit: muda::MenuId,
    autostart: muda::MenuId,
    grant_admin: muda::MenuId,
    edit_config: muda::MenuId,
    reload_config: muda::MenuId,
    ios_sim_route: Option<muda::MenuId>,
    actions: HashMap<muda::MenuId, usize>,
}

impl App {
    /// Создаёт tray icon и меню, запускает event loop до Quit.
    pub fn run(config: Config) -> Result<()> {
        let mut event_loop = EventLoopBuilder::<()>::new().build();
        hide_from_dock(&mut event_loop);

        let (menu, menu_ids) = build_menu(&config)?;
        let any_hosts_active = hosts::any_hosts_route_active(&config);
        let icon = icons::tray_ainv_icon(any_hosts_active);

        let tray = TrayIconBuilder::new()
            .with_icon(icon)
            .with_icon_as_template(false)
            .with_tooltip("AInv Helper")
            .with_menu(Box::new(menu.clone()))
            .build()?;

        let mut app = Self {
            config,
            tray,
            menu,
            menu_ids,
            last_poll: Instant::now(),
            last_tray_hosts_active: Some(any_hosts_active),
            last_ios_sim_active: None,
        };

        app.sync_autostart_menu_item();
        app.sync_ios_sim_route_item();

        let exit_code = event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(250));

            match event {
                Event::NewEvents(StartCause::Init) => {
                    log::info!("AInv Helper started");
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }

            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if app.handle_menu_event(event.id) {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }

            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                log::trace!("Tray event: {event:?}");
            }

            let interval = Duration::from_secs(app.config.poll_interval_secs);
            if app.last_poll.elapsed() >= interval {
                if let Err(err) = app.refresh_tray_indicator() {
                    log::warn!("Tray indicator refresh failed: {err:#}");
                }
                app.sync_ios_sim_route_item();
                app.last_poll = Instant::now();
            }
        });

        log::info!("AInv Helper stopped (exit code {exit_code})");
        Ok(())
    }

    /// Обрабатывает клик по пункту меню. Возвращает `true` для Quit.
    fn handle_menu_event(&mut self, id: muda::MenuId) -> bool {
        if id == self.menu_ids.quit {
            return true;
        }

        if id == self.menu_ids.autostart {
            self.toggle_autostart();
            return false;
        }

        if id == self.menu_ids.edit_config {
            if let Err(err) = config::open_config_in_editor() {
                log::error!("Failed to open config: {err:#}");
            }
            return false;
        }

        if id == self.menu_ids.reload_config {
            self.reload_config();
            return false;
        }

        if id == self.menu_ids.grant_admin {
            if let Err(err) = privileges::request_again() {
                log::error!("Admin privileges request failed: {err:#}");
            }
            return false;
        }

        if let Some(index) = self.menu_ids.actions.get(&id).copied() {
            if let Some(action) = self.config.actions.get(index) {
                let action_type = action.action_type;
                if let Err(err) = actions::execute(action, &self.config) {
                    log::error!("Action '{}' failed: {err:#}", action.label);
                } else if action_type == ActionType::IosSimRoute {
                    self.sync_ios_sim_route_item();
                    if let Err(err) = self.refresh_tray_indicator() {
                        log::warn!("Tray indicator refresh failed: {err:#}");
                    }
                }
            }
        }

        false
    }

    /// Переключает автозапуск и обновляет галочку в меню.
    fn toggle_autostart(&mut self) {
        let enable = !autostart::is_enabled();
        match autostart::set_enabled(enable, config::app_bundle_path().as_ref()) {
            Ok(()) => self.sync_autostart_menu_item(),
            Err(err) => log::error!("Autostart toggle failed: {err:#}"),
        }
    }

    /// Обновляет иконку пункта `toggle ios-sim-route` (✓ / ✗).
    fn sync_ios_sim_route_item(&mut self) {
        let Some(item_id) = &self.menu_ids.ios_sim_route else {
            return;
        };

        let active = hosts::is_ios_sim_route_active();
        if self.last_ios_sim_active == Some(active) {
            return;
        }

        let icon = if active {
            Some(icons::menu_check_green())
        } else {
            Some(icons::menu_cross_red())
        };

        for item in self.menu.items() {
            if item.id() == item_id {
                if let Some(icon_item) = item.as_icon_menuitem() {
                    icon_item.set_icon(icon);
                }
                break;
            }
        }

        self.last_ios_sim_active = Some(active);
        log::debug!("ios-sim-route indicator: {}", if active { "active" } else { "inactive" });
    }

    /// Синхронизирует галочку «Launch at Login» с состоянием LaunchAgent.
    fn sync_autostart_menu_item(&self) {
        for item in self.menu.items() {
            if item.id() == &self.menu_ids.autostart {
                if let Some(check) = item.as_check_menuitem() {
                    let _ = check.set_checked(autostart::is_enabled());
                }
                break;
            }
        }
    }

    /// Перечитывает конфиг с диска и перестраивает меню.
    fn reload_config(&mut self) {
        match config::load_or_create() {
            Ok(new_config) => {
                self.config = new_config;
                if let Err(err) = self.rebuild_menu() {
                    log::error!("Failed to rebuild menu: {err:#}");
                }
                log::info!("Config reloaded");
            }
            Err(err) => log::error!("Config reload failed: {err:#}"),
        }
    }

    /// Пересобирает меню и сбрасывает кэш индикаторов.
    fn rebuild_menu(&mut self) -> Result<()> {
        let (menu, menu_ids) = build_menu(&self.config)?;
        self.menu = menu;
        self.menu_ids = menu_ids;
        self.last_ios_sim_active = None;
        self.last_tray_hosts_active = None;
        self.tray.set_menu(Some(Box::new(self.menu.clone())));
        self.sync_ios_sim_route_item();
        let _ = self.refresh_tray_indicator();
        Ok(())
    }

    /// Обновляет tray icon «AINV» + цветной кружок по состоянию hosts-маршрутов.
    fn refresh_tray_indicator(&mut self) -> Result<()> {
        let any_active = hosts::any_hosts_route_active(&self.config);
        if self.last_tray_hosts_active == Some(any_active) {
            return Ok(());
        }

        self.tray
            .set_icon(Some(icons::tray_ainv_icon(any_active)))?;
        self.last_tray_hosts_active = Some(any_active);
        log::debug!(
            "Tray indicator: {}",
            if any_active { "green" } else { "yellow" }
        );
        Ok(())
    }
}

/// Собирает нативное меню из конфига и системных пунктов.
fn build_menu(config: &Config) -> Result<(Menu, MenuIds)> {
    let menu = Menu::new();

    let mut action_ids = HashMap::new();
    let mut ios_sim_route = None;

    for (index, action) in config.actions.iter().enumerate() {
        match action.action_type {
            ActionType::Header => {
                let item = MenuItem::new(&action.label, false, None);
                menu.append(&item)?;
            }
            ActionType::IosSimRoute => {
                let active = hosts::is_ios_sim_route_active();
                let icon = if active {
                    icons::menu_check_green()
                } else {
                    icons::menu_cross_red()
                };
                let item = IconMenuItem::new(&action.label, true, Some(icon), None);
                let id = item.id().clone();
                ios_sim_route = Some(id.clone());
                action_ids.insert(id, index);
                menu.append(&item)?;
            }
            _ => {
                let item = MenuItem::new(&action.label, true, None);
                let id = item.id().clone();
                action_ids.insert(id, index);
                menu.append(&item)?;
            }
        }
    }

    menu.append(&PredefinedMenuItem::separator())?;

    let edit_config = MenuItem::with_id(MENU_EDIT_CONFIG, "Edit Configuration…", true, None);
    let reload_config = MenuItem::with_id(MENU_RELOAD_CONFIG, "Reload Configuration", true, None);
    menu.append(&edit_config)?;
    menu.append(&reload_config)?;

    menu.append(&PredefinedMenuItem::separator())?;

    let grant_admin = MenuItem::with_id(
        MENU_GRANT_ADMIN,
        "Grant Administrator Access…",
        true,
        None,
    );
    menu.append(&grant_admin)?;

    let autostart = CheckMenuItem::with_id(
        MENU_AUTOSTART,
        "Launch at Login",
        true,
        autostart::is_enabled(),
        None,
    );
    menu.append(&autostart)?;

    menu.append(&PredefinedMenuItem::separator())?;

    let quit = MenuItem::with_id(MENU_QUIT, "Quit AInv Helper", true, None);
    menu.append(&quit)?;

    Ok((
        menu,
        MenuIds {
            quit: quit.id().clone(),
            autostart: autostart.id().clone(),
            grant_admin: grant_admin.id().clone(),
            edit_config: edit_config.id().clone(),
            reload_config: reload_config.id().clone(),
            ios_sim_route,
            actions: action_ids,
        },
    ))
}

/// Скрывает приложение из Dock (`NSApplicationActivationPolicyAccessory`).
#[cfg(target_os = "macos")]
fn hide_from_dock(event_loop: &mut tao::event_loop::EventLoop<()>) {
    use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};

    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    event_loop.set_activate_ignoring_other_apps(false);
}

/// Заглушка для не-macOS платформ.
#[cfg(not(target_os = "macos"))]
fn hide_from_dock(_event_loop: &mut tao::event_loop::EventLoop<()>) {}
