mod automations;
mod commands;
mod config;
mod lcu;
mod modules;
mod state;
mod switcher;
mod tray;
mod updater;

use std::sync::Arc;

use tauri::{Emitter, Manager};

use crate::config::Config;
use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let app = window.app_handle();
                let state = app.state::<Arc<AppState>>();

                let (close_to_tray, ask_on_close) = state
                    .config
                    .try_read()
                    .map(|c| (c.ui.close_to_tray, c.ui.ask_on_close))
                    .unwrap_or((true, true));
                if ask_on_close {
                    api.prevent_close();
                    let _ = window.emit("prowler://close-requested", ());
                } else if close_to_tray {
                    api.prevent_close();
                    let win = window.clone();

                    tauri::async_runtime::spawn(async move {
                        let _ = win.destroy();
                    });
                } else {
                    app.exit(0);
                }
            }
        })
        .setup(|app| {
            let config_dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("prowler"));
            let config_path = config_dir.join("config.json");
            let config = Config::load_or_default(&config_path);

            let vault_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("prowler"))
                .join("vault");

            let state = Arc::new(AppState::new(config, config_path, vault_dir));
            app.manage(state.clone());

            let handle = app.handle().clone();
            let monitor_state = state.clone();
            tauri::async_runtime::spawn(async move {
                commands::connection_monitor(handle, monitor_state).await;
            });

            let ws_state = state.clone();
            tauri::async_runtime::spawn(async move {
                lcu::ws::run_event_bus(ws_state).await;
            });

            let handle = app.handle().clone();
            let automations_state = state.clone();
            tauri::async_runtime::spawn(async move {
                automations::sync(&handle, &automations_state).await;
            });

            #[cfg(desktop)]
            {
                tray::create_tray(app.handle())?;
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
                app.manage(updater::PendingUpdate(tokio::sync::Mutex::new(None)));
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    updater::auto_check(handle, state).await;
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_connection_status,
            commands::quit,
            commands::get_config,
            commands::set_config,
            commands::list_accounts,
            commands::save_current_account,
            commands::switch_account,
            commands::rename_account,
            commands::delete_account,
            automations::set_automation,
            automations::get_running_automations,
            modules::champ_select::list_champions,
            modules::icons::set_profile_icon,
            modules::icons::set_client_icon,
            modules::backgrounds::set_background,
            modules::status::set_status_message,
            modules::badges::set_badges,
            modules::reveal::reveal_lobby,
            modules::dodge::dodge,
            modules::restart::restart_ux,
            modules::chat::set_chat_offline,
            modules::friends::count_friends,
            modules::friends::remove_all_friends,
            modules::runes::get_rune_trees,
            modules::runes::apply_rune_page,
            updater::check_update,
            updater::install_update,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                api.prevent_exit();
            }
        });
}
