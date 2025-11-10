#![warn(clippy::all, rust_2018_idioms)]

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
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app: App = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        let system = app.system.clone();
        let ctx = cc.egui_ctx.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs_f32(SYSTEM_REFRESH_INTERVAL_SECONDS));
                system
                    .lock()
                    .unwrap()
                    .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                ctx.request_repaint();
            }
        });

        app
    }
}

impl eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        processes::update(self, ctx);
    }

    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
