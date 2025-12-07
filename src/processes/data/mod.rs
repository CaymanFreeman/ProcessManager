mod extraction;
mod sorting;
mod tree;

use crate::app;
use extraction::extract_processes_info;

pub use extraction::extract_name;
pub use extraction::extract_path;
pub use sorting::SortCategory;
pub use sorting::SortDirection;
pub use sorting::SortMethod;
pub use tree::ProcessTree;

pub struct ProcessInfo {
    pub child_depth: usize,
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
    let (system, user_input) = (app.system(), app.user_input());
    let (Ok(system), Ok(user_input)) = (system.read(), user_input.read()) else {
        return Vec::new();
    };

    let mut processes: Vec<&sysinfo::Process> = system.processes().values().collect();
    let users = sysinfo::Users::new_with_refreshed_list();
    let cpu_count = system.cpus().len();

    filter_thread_processes(user_input.show_thread_processes(), &mut processes);

    let mut processes_info = if user_input.hierarchical_view() {
        let process_tree = ProcessTree::build(&processes);
        let (processes, indentations) = process_tree.flattened();
        extract_processes_info(&processes, indentations, &users, cpu_count)
    } else {
        let mut processes_info = extract_processes_info(
            &processes,
            vec![0_usize; processes.len()],
            &users,
            cpu_count,
        );
        user_input.sort_method().sort(&mut processes_info);
        processes_info
    };

    filter_user_input(user_input.process_filter(), &mut processes_info);

    processes_info
}

fn filter_thread_processes(show_thread_processes: bool, processes: &mut Vec<&sysinfo::Process>) {
    if show_thread_processes {
        return;
    }

    processes.retain(|process| process.thread_kind().is_none());
}

fn filter_user_input(process_filter: &str, processes_info: &mut Vec<ProcessInfo>) {
    if process_filter.is_empty() {
        return;
    }

    processes_info.retain(|process_info| {
        process_info.name.contains(process_filter)
            || process_info.user.contains(process_filter)
            || process_info.path.contains(process_filter)
    });
}
