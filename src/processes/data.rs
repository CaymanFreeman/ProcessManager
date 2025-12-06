use crate::app;

const UNKNOWN_PROCESS_PATH: &str = "-";
const UNKNOWN_PROCESS_NAME: &str = "-";
const UNKNOWN_USER: &str = "-";

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq)]
pub enum SortCategory {
    Id,
    Name,
    User,
    Memory,
    Cpu,
    DiskRead,
    DiskWrite,
    Status,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SortMethod {
    pub(crate) category: SortCategory,
    pub(crate) direction: SortDirection,
}

impl SortMethod {
    pub fn sort(&self, processes_info: &mut [ProcessInfo]) {
        match self.category {
            SortCategory::Id => processes_info.sort_by(|a, b| a.id.cmp(&b.id)),
            SortCategory::Name => processes_info.sort_by(|a, b| a.name.cmp(&b.name)),
            SortCategory::User => processes_info.sort_by(|a, b| a.user.cmp(&b.user)),
            SortCategory::Memory => processes_info.sort_by(|a, b| a.memory.cmp(&b.memory)),
            SortCategory::Cpu => processes_info.sort_by(|a, b| a.cpu.cmp(&b.cpu)),
            SortCategory::DiskRead => processes_info.sort_by(|a, b| a.disk_read.cmp(&b.disk_read)),
            SortCategory::DiskWrite => {
                processes_info.sort_by(|a, b| a.disk_write.cmp(&b.disk_write));
            }
            SortCategory::Status => processes_info.sort_by(|a, b| a.status.cmp(&b.status)),
        }

        if matches!(self.direction, SortDirection::Descending) {
            processes_info.reverse();
        }
    }

    pub fn toggle_direction(&mut self) {
        match self.direction {
            SortDirection::Ascending => self.direction = SortDirection::Descending,
            SortDirection::Descending => self.direction = SortDirection::Ascending,
        }
    }
}

impl Default for SortMethod {
    fn default() -> Self {
        Self {
            category: SortCategory::Cpu,
            direction: SortDirection::Descending,
        }
    }
}

pub struct ProcessInfo {
    pub id: u32,
    pub name: String,
    pub user: String,
    pub memory: u64,
    pub cpu: String,
    pub disk_read: u64,
    pub disk_write: u64,
    pub path: String,
    pub status: String,
}

pub fn prepare_processes(app: &app::App) -> Vec<ProcessInfo> {
    if let Ok(system) = app.system().lock() {
        let mut processes: Vec<&sysinfo::Process> = system.processes().values().collect();
        let users = sysinfo::Users::new_with_refreshed_list();
        let cpu_count = system.cpus().len();

        filter_thread_processes(app, &mut processes);
        let mut processes_info = extract_processes_info(&processes, &users, cpu_count);
        sort_processes_info(app, &mut processes_info);

        processes_info
    } else {
        Vec::new()
    }
}

fn filter_thread_processes(app: &app::App, processes: &mut Vec<&sysinfo::Process>) {
    if !app.show_thread_processes() {
        processes.retain(|process| process.thread_kind().is_none());
    }
}

fn extract_processes_info(
    processes: &[&sysinfo::Process],
    users: &sysinfo::Users,
    cpu_count: usize,
) -> Vec<ProcessInfo> {
    processes
        .iter()
        .map(|process| extract_info(process, users, cpu_count))
        .collect()
}

fn sort_processes_info(app: &app::App, processes_info: &mut [ProcessInfo]) {
    app.sort_method().sort(processes_info);
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
