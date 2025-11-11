use crate::processes;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const SYSTEM_REFRESH_INTERVAL_SECONDS: f32 = 1.0;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    pub system: Arc<Mutex<sysinfo::System>>,
    #[serde(skip)]
    pub selected_pid: Option<sysinfo::Pid>,
    #[serde(skip)]
    pub process_filter: String,
    pub show_thread_processes: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            system: Arc::new(Mutex::new(sysinfo::System::new_all())),
            selected_pid: None,
            process_filter: String::new(),
            show_thread_processes: false,
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
        thread::spawn(move || system_refresh_loop(&system, &ctx));

        app
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

fn system_refresh_loop(system: &Arc<Mutex<sysinfo::System>>, ctx: &egui::Context) -> ! {
    loop {
        thread::sleep(Duration::from_secs_f32(SYSTEM_REFRESH_INTERVAL_SECONDS));
        system
            .lock()
            .expect("Failed to acquire system lock")
            .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
        ctx.request_repaint();
    }
}
