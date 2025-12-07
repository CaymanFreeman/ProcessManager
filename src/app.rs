use crate::processes;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

const SYSTEM_REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    #[serde(skip)]
    system: Arc<RwLock<sysinfo::System>>,

    #[serde(skip)]
    selected_pid: Option<u32>,

    user_input: Arc<RwLock<processes::UserInput>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            system: Arc::new(RwLock::new(sysinfo::System::new_all())),
            user_input: Arc::new(RwLock::new(processes::UserInput::default())),
            selected_pid: None,
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

    pub(crate) fn system(&self) -> Arc<RwLock<sysinfo::System>> {
        self.system.clone()
    }

    pub(crate) fn user_input(&self) -> Arc<RwLock<processes::UserInput>> {
        self.user_input.clone()
    }

    fn system_refresh_loop(system: &Arc<RwLock<sysinfo::System>>, ctx: &egui::Context) -> ! {
        loop {
            thread::sleep(SYSTEM_REFRESH_INTERVAL);
            let Ok(mut system) = system.write() else {
                continue;
            };
            system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
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
