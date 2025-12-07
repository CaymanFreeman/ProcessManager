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
    user_input: Arc<RwLock<processes::UserInput>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            system: Arc::new(RwLock::new(sysinfo::System::new_all())),
            user_input: Arc::new(RwLock::new(processes::UserInput::default())),
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
        let user_input = app.user_input.clone();
        let ctx = cc.egui_ctx.clone();
        thread::spawn(move || Self::system_refresh_loop(&system, &user_input, &ctx));

        app
    }

    pub(crate) fn system(&self) -> Arc<RwLock<sysinfo::System>> {
        self.system.clone()
    }

    pub(crate) fn user_input(&self) -> Arc<RwLock<processes::UserInput>> {
        self.user_input.clone()
    }

    fn system_refresh_loop(
        system: &Arc<RwLock<sysinfo::System>>,
        user_input: &Arc<RwLock<processes::UserInput>>,
        ctx: &egui::Context,
    ) -> ! {
        loop {
            let Ok(user_input) = user_input.read() else {
                continue;
            };

            if !user_input.continue_refreshing() {
                continue;
            }

            drop(user_input); // The UI will use this lock to repaint

            if let Ok(mut system) = system.write() {
                system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            }
            ctx.request_repaint();

            thread::sleep(SYSTEM_REFRESH_INTERVAL);
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
