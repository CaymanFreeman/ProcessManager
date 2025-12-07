use crate::processes::data;

const UNKNOWN_PROCESS_PATH: &str = "-";
const UNKNOWN_PROCESS_NAME: &str = "-";
const UNKNOWN_USER: &str = "-";

pub fn extract_processes_info(
    processes: &[&sysinfo::Process],
    indentations: Vec<usize>,
    users: &sysinfo::Users,
    cpu_count: usize,
) -> Vec<data::ProcessInfo> {
    processes
        .iter()
        .zip(indentations)
        .map(|(process, child_depth)| extract_info(process, child_depth, users, cpu_count))
        .collect()
}

fn extract_info(
    process: &sysinfo::Process,
    child_depth: usize,
    users: &sysinfo::Users,
    cpu_count: usize,
) -> data::ProcessInfo {
    data::ProcessInfo {
        child_depth,
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

fn extract_memory(process: &sysinfo::Process) -> u64 {
    process.memory()
}

fn extract_cpu(process: &sysinfo::Process, cpu_count: usize) -> String {
    format!("{:.2}%", process.cpu_usage() / cpu_count as f32)
}

fn extract_disk_read(process: &sysinfo::Process) -> u64 {
    process.disk_usage().read_bytes
}

fn extract_disk_write(process: &sysinfo::Process) -> u64 {
    process.disk_usage().written_bytes
}

pub fn extract_path(process: &sysinfo::Process) -> Option<&str> {
    let path = process.exe()?;
    path.to_str()
}

fn extract_status(process: &sysinfo::Process) -> String {
    process.status().to_string()
}
