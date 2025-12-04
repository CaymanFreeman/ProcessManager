const UNKNOWN_PROCESS_PATH: &str = "-";
const UNKNOWN_PROCESS_NAME: &str = "-";
const UNKNOWN_USER: &str = "-";

pub struct ProcessInfo {
    pub id: String,
    pub name: String,
    pub user: String,
    pub memory: String,
    pub cpu: String,
    pub disk_read: String,
    pub disk_write: String,
    pub path: String,
    pub status: String,
}

fn prepare_info(
    process: &sysinfo::Process,
    users: &sysinfo::Users,
    cpu_count: usize,
) -> ProcessInfo {
    ProcessInfo {
        id: extract_id(process),
        name: extract_name(process),
        user: extract_user(process, users),
        memory: extract_memory(process),
        cpu: extract_cpu(process, cpu_count),
        disk_read: extract_disk_read(process),
        disk_write: extract_disk_write(process),
        path: extract_path(process),
        status: extract_status(process),
    }
}

fn extract_id(process: &sysinfo::Process) -> String {
    process.pid().to_string()
}

fn extract_name(process: &sysinfo::Process) -> String {
    process
        .name()
        .to_str()
        .unwrap_or(UNKNOWN_PROCESS_NAME)
        .to_owned()
}

fn extract_user(process: &sysinfo::Process, users: &sysinfo::Users) -> String {
    process
        .user_id()
        .and_then(|uid| users.get_user_by_id(uid))
        .map(|user| user.name())
        .unwrap_or(UNKNOWN_USER)
        .to_owned()
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

fn extract_path(process: &sysinfo::Process) -> String {
    process
        .exe()
        .and_then(|path| path.to_str())
        .unwrap_or(UNKNOWN_PROCESS_PATH)
        .to_owned()
}

fn extract_status(process: &sysinfo::Process) -> String {
    process.status().to_string()
}
