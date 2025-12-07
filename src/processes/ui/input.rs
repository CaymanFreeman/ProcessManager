use crate::processes;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserInput {
    #[serde(skip)]
    selected_pid: Option<u32>,

    show_thread_processes: bool,
    hierarchical_view: bool,
    process_filter: String,
    sort_method: processes::SortMethod,
    continue_refreshing: bool,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            selected_pid: None,
            show_thread_processes: false,
            hierarchical_view: true,
            process_filter: String::new(),
            sort_method: Default::default(),
            continue_refreshing: true,
        }
    }
}

impl UserInput {
    pub(crate) fn selected_pid(&self) -> Option<u32> {
        self.selected_pid
    }

    pub(crate) fn set_selected_pid(&mut self, pid: Option<u32>) {
        self.selected_pid = pid;
    }

    pub(crate) fn process_filter(&self) -> &str {
        &self.process_filter
    }

    pub(crate) fn clear_process_filter(&mut self) {
        self.process_filter = String::new();
    }

    pub(crate) fn process_filter_mut(&mut self) -> &mut String {
        &mut self.process_filter
    }

    pub(crate) fn hierarchical_view(&self) -> bool {
        self.hierarchical_view
    }

    pub(crate) fn hierarchical_view_mut(&mut self) -> &mut bool {
        &mut self.hierarchical_view
    }

    pub(crate) fn show_thread_processes(&self) -> bool {
        self.show_thread_processes
    }

    pub(crate) fn show_thread_processes_mut(&mut self) -> &mut bool {
        &mut self.show_thread_processes
    }

    pub(crate) fn sort_method(&self) -> &processes::SortMethod {
        &self.sort_method
    }

    pub(crate) fn sort_method_mut(&mut self) -> &mut processes::SortMethod {
        &mut self.sort_method
    }

    pub(crate) fn continue_refreshing(&self) -> bool {
        self.continue_refreshing
    }

    pub(crate) fn set_continue_refreshing(&mut self, continue_refreshing: bool) {
        self.continue_refreshing = continue_refreshing;
    }
}
