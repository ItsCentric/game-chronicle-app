use sysinfo::{Pid, System};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::{NotificationExt, PermissionState};

use crate::database::get_executable_details;
use crate::{helpers, Error};

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub path: String,
    pub run_time: u64,
}

#[derive(Debug, serde::Serialize)]
struct GameStoppedPayload {
    executable_name: Option<String>,
    game_id: Option<i32>,
    minutes_played: i32,
}

impl Process {
    fn new(pid: usize, sys: &System) -> Result<Process, Error> {
        let process = sys.process(Pid::from(pid));
        match process {
            Some(p) => {
                let name = p.name().to_string();
                let path = match p.exe() {
                    Some(path) => path.to_string_lossy().to_string(),
                    None => "".to_string(),
                };
                Ok(Process {
                    name,
                    pid: pid.try_into().unwrap(),
                    path,
                    run_time: p.run_time(),
                })
            }
            None => Err(Error::ProcessNotFound),
        }
    }
}

type ProcessMap = HashMap<PathBuf, Process>;
pub struct ProcessMonitor {
    previous_running_processes: ProcessMap,
}

impl ProcessMonitor {
    pub fn new() -> ProcessMonitor {
        ProcessMonitor {
            previous_running_processes: HashMap::new(),
        }
    }

    pub fn monitor_processes(
        &mut self,
        paths_to_monitor: Vec<PathBuf>,
        app: &tauri::AppHandle,
    ) -> Result<(), Error> {
        let mut running_processes = get_running_processes()?;
        if !paths_to_monitor.is_empty() {
            running_processes = filter_process_map(running_processes, paths_to_monitor);
        }
        for (path, process) in &self.previous_running_processes {
            if !running_processes.contains_key(path) {
                let minutes_played = process.run_time / 60;
                if minutes_played < 1 {
                    continue;
                }
                let data_dir = helpers::get_app_data_directory(app)?;
                let conn = rusqlite::Connection::open(data_dir.join("logs.db"))?;
                match get_executable_details(&conn, &process.name) {
                    Ok(details) => {
                        app.emit(
                            "game-stopped",
                            &GameStoppedPayload {
                                executable_name: None,
                                game_id: Some(details.game_id),
                                minutes_played: minutes_played as i32,
                            },
                        )?;
                    }
                    Err(e) => match e {
                        Error::Rusqlite(rusqlite::Error::QueryReturnedNoRows) => {
                            app.emit(
                                "game-stopped",
                                &GameStoppedPayload {
                                    executable_name: Some(process.name.clone()),
                                    game_id: None,
                                    minutes_played: minutes_played as i32,
                                },
                            )?;
                        }
                        _ => return Err(e.into()),
                    },
                };
                match app.get_webview_window("main") {
                    Some(window) => {
                        window.show()?;
                    }
                    None => {
                        let mut notification_permission_state =
                            app.notification().permission_state().unwrap();
                        if notification_permission_state != PermissionState::Granted {
                            notification_permission_state =
                                app.notification().request_permission().unwrap();
                            if notification_permission_state != PermissionState::Granted {
                                return Ok(());
                            }
                        }
                        app
                            .notification()
                            .builder()
                            .title("Game stopped")
                            .body("A game was detected to have stopped but the main window could not be opened automatically.")
                            .show()
                            .unwrap();
                    }
                }
            }
        }
        self.previous_running_processes = running_processes;
        Ok(())
    }
}

pub fn get_running_processes() -> Result<ProcessMap, Error> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let processes = sys.processes();
    let mut running_processes = HashMap::new();
    for (pid, _) in processes {
        let pid = pid.as_u32();
        match Process::new(pid.try_into().unwrap(), &sys) {
            Ok(process) => running_processes.insert(PathBuf::from(&process.path), process),
            Err(_) => continue,
        };
    }
    Ok(running_processes)
}

pub fn filter_process_map(process_map: ProcessMap, paths: Vec<PathBuf>) -> ProcessMap {
    let filtered_map = process_map
        .into_iter()
        .filter(|(path, _)| paths.contains(path))
        .collect();
    filtered_map
}
