use crate::processes;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SYSTEM_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[serde(skip)]
    selected_pid: Option<u32>,

    #[serde(skip)]
    process_filter: String,

    show_thread_processes: bool,
    sort_method: processes::SortMethod,
}

impl Default for App {
    fn default() -> Self {
        Self {
            system: Arc::new(Mutex::new(sysinfo::System::new_all())),
            selected_pid: None,
            process_filter: String::new(),
            show_thread_processes: false,
            sort_method: processes::SortMethod::default(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        let system = app.system.clone();
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || Self::system_refresh_loop(&system, &ctx));

        app
    }

    pub(crate) fn selected_pid(&self) -> Option<u32> {
        self.selected_pid
    }

    pub(crate) fn set_selected_pid(&mut self, pid: Option<u32>) {
        self.selected_pid = pid;
    }

    pub(crate) fn process_filter(&self) -> &str {
        &self.process_filter
    }

    pub(crate) fn process_filter_mut(&mut self) -> &mut String {
        &mut self.process_filter
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

    pub(crate) fn system(&self) -> Arc<Mutex<sysinfo::System>> {
        self.system.clone()
    }

    fn system_refresh_loop(system: &Arc<Mutex<sysinfo::System>>, ctx: &egui::Context) -> ! {
        loop {
            thread::sleep(SYSTEM_REFRESH_INTERVAL);
            if let Ok(mut sys) = system.lock() {
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            }
            ctx.request_repaint();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        processes::update(self, ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
