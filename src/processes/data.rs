use std::sync::{Arc, Mutex};

const UNKNOWN_PROCESS_PATH: &str = "-";
const UNKNOWN_PROCESS_NAME: &str = "-";
const UNKNOWN_USER: &str = "-";

pub struct ProcessInfo {
    pub id: u32,
    pub name: String,
    pub user: String,
    pub memory: String,
    pub cpu: String,
    pub disk_read: String,
    pub disk_write: String,
    pub path: String,
    pub status: String,
}

pub fn prepare_processes(system: &Arc<Mutex<sysinfo::System>>) -> Vec<ProcessInfo> {
    if let Ok(system) = system.lock() {
        let processes: Vec<&sysinfo::Process> = system.processes().values().collect();
        let users = sysinfo::Users::new_with_refreshed_list();
        let cpu_count = system.cpus().len();
        let processes_info: Vec<ProcessInfo> = processes
            .iter()
            .map(|process| extract_info(process, &users, cpu_count))
            .collect();
        processes_info
    } else {
        Vec::new()
    }
}

fn extract_info(
    process: &sysinfo::Process,
    users: &sysinfo::Users,
    cpu_count: usize,
) -> ProcessInfo {
    ProcessInfo {
        id: extract_id(process),
        name: extract_name(process)
            .unwrap_or(UNKNOWN_PROCESS_NAME)
            .to_owned(),
        user: extract_user(process, users)
            .unwrap_or(UNKNOWN_USER)
            .to_owned(),
        memory: extract_memory(process),
        cpu: extract_cpu(process, cpu_count),
        disk_read: extract_disk_read(process),
        disk_write: extract_disk_write(process),
        path: extract_path(process)
            .unwrap_or(UNKNOWN_PROCESS_PATH)
            .to_owned(),
        status: extract_status(process),
    }
}

fn extract_id(process: &sysinfo::Process) -> u32 {
    process.pid().as_u32()
}

pub fn extract_name(process: &sysinfo::Process) -> Option<&str> {
    process.name().to_str()
}

fn extract_user<'a>(process: &sysinfo::Process, users: &'a sysinfo::Users) -> Option<&'a str> {
    process
        .user_id()
        .and_then(|uid| users.get_user_by_id(uid))
        .map(|user| user.name())
}

fn extract_memory(process: &sysinfo::Process) -> String {
    bytesize::ByteSize(process.memory()).to_string()
}

fn extract_cpu(process: &sysinfo::Process, cpu_count: usize) -> String {
    format!("{:.2}%", process.cpu_usage() / cpu_count as f32)
}

fn extract_disk_read(process: &sysinfo::Process) -> String {
    bytesize::ByteSize(process.disk_usage().read_bytes).to_string()
}

fn extract_disk_write(process: &sysinfo::Process) -> String {
    bytesize::ByteSize(process.disk_usage().written_bytes).to_string()
}

pub fn extract_path(process: &sysinfo::Process) -> Option<&str> {
    process.exe().and_then(|path| path.to_str())
}

fn extract_status(process: &sysinfo::Process) -> String {
    process.status().to_string()
}
