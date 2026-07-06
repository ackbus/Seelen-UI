use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::PhysicalMonitor};

use crate::{
    app::emit_to_webviews, error::Result, modules::monitors::MonitorManager,
    windows_api::MonitorEnumerator,
};

fn get_monitor_manager() -> &'static MonitorManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        let initial = _get_connected_monitors();
        if let Ok(monitors) = initial {
            log::debug!("Initial monitors: {monitors:#?}");
        }
        MonitorManager::subscribe(|_event| {
            if let Ok(monitors) = _get_connected_monitors() {
                log::debug!("Monitors changed: {monitors:#?}");
                emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors);
            }
        });

        crate::get_tokio_handle().spawn(async {
            let mut timer = tokio::time::interval(std::time::Duration::from_secs(3));
            loop {
                timer.tick().await;
                let sentinel = std::env::temp_dir().join("seelen_trigger_monitors");
                if !sentinel.exists() {
                    continue;
                }
                let _ = std::fs::remove_file(&sentinel);
                log::debug!("Sentinel trigger detected");
                if let Ok(monitors) = _get_connected_monitors() {
                    emit_to_webviews(SeelenEvent::SystemMonitorsChanged, monitors);
                }
            }
        });
    });
    MonitorManager::instance()
}

pub fn _get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    let mut monitors = Vec::new();
    for m in MonitorEnumerator::enumerate_win32()? {
        if let Ok(pm) = m.try_into() {
            monitors.push(pm);
        }
    }
    Ok(monitors)
}

#[tauri::command(async)]
pub fn get_connected_monitors() -> Result<Vec<PhysicalMonitor>> {
    get_monitor_manager();
    _get_connected_monitors()
}
