use sysinfo::{Pid, System};
use tauri::Manager;

use crate::database::get_executable_details;
use crate::Error;

use std::any::Any;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Process {
    pub name: String,
    pub pid: u32,
    pub path: String,
    pub create_time: u64,
}

#[derive(Debug, serde::Serialize)]
struct GameStoppedPayload {
    name: String,
    igdb_id: i32,
    minutes_played: i32,
}

impl Process {
    fn new(pid: usize, sys: &System) -> Result<Process, Error> {
        let process = sys.process(Pid::from(pid));
        match process {
            Some(p) => {
                let name = p.name().to_string();
                let path = p
                    .exe()
                    .expect("Path to exe to exist")
                    .to_string_lossy()
                    .to_string();
                let create_time = p.start_time();
                Ok(Process {
                    name,
                    pid: pid.try_into().unwrap(),
                    path,
                    create_time,
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
                println!("Process {} has been stopped", process.name);
                let minutes_played = process.create_time / 60;
                let conn = rusqlite::Connection::open("logs.db")?;
                match get_executable_details(&conn, &process.name) {
                    Ok(details) => {
                        app.emit_all(
                            "game-stopped",
                            &GameStoppedPayload {
                                name: details.name,
                                igdb_id: details.igdb_id,
                                minutes_played: details.minutes_played + minutes_played as i32,
                            },
                        )?;
                    }
                    Err(e) => {
                        if e.type_id() == rusqlite::Error::QueryReturnedNoRows.type_id() {
                            app.emit_all(
                                "game-stopped",
                                &GameStoppedPayload {
                                    name: process.name.clone(),
                                    igdb_id: -1,
                                    minutes_played: minutes_played as i32,
                                },
                            )?;
                        } else {
                            return Err(e);
                        }
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
