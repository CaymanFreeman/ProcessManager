use crate::{app, processes};
use processes::data;
use std::ops::RangeInclusive;

const HEADER_TEXT_SIZE: f32 = 12.0;
const HEADER_HEIGHT: f32 = 25.0;
const ROW_HEIGHT: f32 = 18.0;
const CONTROL_PANEL_HEIGHT: f32 = 30.0;
const LARGE_COLUMN_WIDTH: f32 = 250.0;
const SMALL_COLUMNS_WIDTH: f32 = 65.0;
const COLUMN_WIDTH_RANGE: RangeInclusive<f32> = 65.0..=500.0;

const CLIPBOARD_SYMBOL: &str = "📋";
const PLAY_SYMBOL: &str = "▶";
const PAUSE_SYMBOL: &str = "⏸";
const REFRESH_SYMBOL: &str = "⟳";

const BLANK_PROCESS_PATH: &str = "";
const BLANK_PROCESS_NAME: &str = "";

const ASCENDING_SYMBOL: &str = "⏶";
const DESCENDING_SYMBOL: &str = "⏷";

#[derive(serde::Deserialize, serde::Serialize)]
pub struct UserInput {
    #[serde(skip)]
    selected_pid: Option<u32>,

    show_thread_processes: bool,
    process_filter: String,
    sort_method: processes::SortMethod,
    continue_refreshing: bool,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            selected_pid: None,
            show_thread_processes: false,
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

    pub(crate) fn continue_refreshing(&self) -> bool {
        self.continue_refreshing
    }

    pub(crate) fn set_continue_refreshing(&mut self, continue_refreshing: bool) {
        self.continue_refreshing = continue_refreshing;
    }
}

pub fn update(app: &app::App, ctx: &egui::Context) {
    egui::TopBottomPanel::top("options_bar").show(ctx, |ui| {
        update_options_panel(app, ui);
    });
    egui::TopBottomPanel::bottom("control_bar")
        .exact_height(CONTROL_PANEL_HEIGHT)
        .show(ctx, |ui| {
            update_control_bar(app, ctx, ui);
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        update_table(app, ui);
    });
}

fn update_options_panel(app: &app::App, ui: &mut egui::Ui) {
    let (system, user_input) = (app.system(), app.user_input());
    let Ok(mut user_input) = user_input.write() else {
        return;
    };
    ui.horizontal(|ui| {
        if ui.button(PLAY_SYMBOL).clicked() {
            user_input.set_continue_refreshing(true);
        }

        if ui.button(PAUSE_SYMBOL).clicked() {
            user_input.set_continue_refreshing(false);
        }

        if ui.button(REFRESH_SYMBOL).clicked() {
            if let Ok(mut system) = system.write() {
                system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            }
            ui.ctx().request_repaint();
        }

        ui.separator();

        ui.add(
            egui::TextEdit::singleline(user_input.process_filter_mut())
                .hint_text("Filter by name, user, or path"),
        );

        ui.separator();

        ui.checkbox(user_input.show_thread_processes_mut(), "Include Threads");
    });
}

fn update_control_bar(app: &app::App, ctx: &egui::Context, ui: &mut egui::Ui) {
    let (system, user_input) = (app.system(), app.user_input());
    let (Ok(system), Ok(user_input)) = (system.read(), user_input.read()) else {
        return;
    };

    let Some(selected_pid) = user_input.selected_pid() else {
        return;
    };

    let Some(process) = system
        .processes()
        .get(&sysinfo::Pid::from_u32(selected_pid))
    else {
        return;
    };

    ui.horizontal_centered(|ui| {
        if ui.button("Terminate").clicked() {
            process.kill_with(sysinfo::Signal::Term);
        }

        //ui.separator();

        if ui.button("Kill").clicked() {
            process.kill_with(sysinfo::Signal::Kill);
        }

        ui.separator();

        if let Some(name) = data::extract_name(process) {
            if ui.button(CLIPBOARD_SYMBOL).clicked() {
                ctx.copy_text(
                    data::extract_name(process)
                        .unwrap_or(BLANK_PROCESS_NAME)
                        .to_owned(),
                );
            }
            ui.label(name);
            ui.separator();
        }

        if ui.button(CLIPBOARD_SYMBOL).clicked() {
            ctx.copy_text(selected_pid.to_string());
        }
        ui.label(selected_pid.to_string());

        ui.separator();

        if let Some(path) = data::extract_path(process) {
            if ui.button(CLIPBOARD_SYMBOL).clicked() {
                ctx.copy_text(
                    data::extract_path(process)
                        .unwrap_or(BLANK_PROCESS_PATH)
                        .to_owned(),
                );
            }
            ui.label(path);
            ui.separator();
        }
    });

    ui.separator();
}

fn header_name_label(text: &str, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(text).font(egui::FontId::proportional(HEADER_TEXT_SIZE)));
}

fn header_sort_label(sort_direction: &data::SortDirection, ui: &mut egui::Ui) {
    match sort_direction {
        data::SortDirection::Ascending => {
            ui.label(
                egui::RichText::new(ASCENDING_SYMBOL)
                    .font(egui::FontId::proportional(HEADER_TEXT_SIZE)),
            );
        }
        data::SortDirection::Descending => {
            ui.label(
                egui::RichText::new(DESCENDING_SYMBOL)
                    .font(egui::FontId::proportional(HEADER_TEXT_SIZE)),
            );
        }
    }
}

fn response_primary_clicked(response: &egui::Response) -> bool {
    response.hovered() && response.ctx.input(|i| i.pointer.primary_clicked())
}

fn header_cell(
    text: &str,
    header_category: Option<data::SortCategory>,
    current_sort_method: &mut data::SortMethod,
    ui: &mut egui::Ui,
) {
    ui.style_mut().interaction.selectable_labels = false;

    let Some(sort_category) = header_category else {
        ui.horizontal_centered(|ui| {
            header_name_label(text, ui);
        });
        return;
    };

    if current_sort_method.category == sort_category {
        ui.horizontal_centered(|ui| {
            header_name_label(text, ui);
            header_sort_label(&current_sort_method.direction, ui);
        });
    } else {
        ui.horizontal_centered(|ui| {
            header_name_label(text, ui);
        });
    }

    if ui.response().hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    if response_primary_clicked(&ui.response()) {
        if current_sort_method.category == sort_category {
            current_sort_method.toggle_direction();
        } else {
            current_sort_method.category = sort_category;
            current_sort_method.direction = data::SortDirection::Ascending;
        }
    }
}

fn body_cell(text: &str, ui: &mut egui::Ui) {
    ui.style_mut().interaction.selectable_labels = false;
    ui.label(text);
}

fn format_bytes(bytes: u64) -> String {
    bytesize::ByteSize(bytes).to_string()
}

fn large_column() -> egui_extras::Column {
    egui_extras::Column::exact(LARGE_COLUMN_WIDTH)
        .clip(true)
        .range(COLUMN_WIDTH_RANGE)
        .resizable(true)
}

fn small_column() -> egui_extras::Column {
    egui_extras::Column::exact(SMALL_COLUMNS_WIDTH)
        .clip(true)
        .range(COLUMN_WIDTH_RANGE)
        .resizable(true)
}

fn update_table(app: &app::App, ui: &mut egui::Ui) {
    let processes_info = data::prepare_processes(app);
    let user_input = app.user_input();
    let Ok(mut user_input) = user_input.write() else {
        return;
    };

    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .column(large_column())
        .columns(small_column(), 6)
        .column(large_column())
        .column(small_column())
        .header(HEADER_HEIGHT, |mut header_row| {
            let sort_method = user_input.sort_method_mut();
            header_row.col(|ui| header_cell("Name", None, sort_method, ui));
            header_row.col(|ui| header_cell("ID", Some(data::SortCategory::Id), sort_method, ui));
            header_row.col(|ui| header_cell("User", None, sort_method, ui));
            header_row
                .col(|ui| header_cell("Memory", Some(data::SortCategory::Memory), sort_method, ui));
            header_row.col(|ui| header_cell("CPU", Some(data::SortCategory::Cpu), sort_method, ui));
            header_row.col(|ui| {
                header_cell(
                    "Disk Read",
                    Some(data::SortCategory::DiskRead),
                    sort_method,
                    ui,
                );
            });
            header_row.col(|ui| {
                header_cell(
                    "Disk Write",
                    Some(data::SortCategory::DiskWrite),
                    sort_method,
                    ui,
                );
            });
            header_row.col(|ui| header_cell("Path", None, sort_method, ui));
            header_row
                .col(|ui| header_cell("Status", Some(data::SortCategory::Status), sort_method, ui));
        })
        .body(|mut body_rows| {
            for process_info in processes_info {
                body_rows.row(ROW_HEIGHT, |mut row| {
                    row.set_selected(user_input.selected_pid() == Some(process_info.id));

                    row.col(|ui| body_cell(&process_info.name, ui));
                    row.col(|ui| body_cell(process_info.id.to_string().as_str(), ui));
                    row.col(|ui| body_cell(&process_info.user, ui));
                    row.col(|ui| body_cell(format_bytes(process_info.memory).as_str(), ui));
                    row.col(|ui| body_cell(&process_info.cpu, ui));
                    row.col(|ui| body_cell(format_bytes(process_info.disk_read).as_str(), ui));
                    row.col(|ui| body_cell(format_bytes(process_info.disk_write).as_str(), ui));
                    row.col(|ui| body_cell(&process_info.path, ui));
                    row.col(|ui| body_cell(&process_info.status, ui));

                    if response_primary_clicked(&row.response()) {
                        user_input.set_selected_pid(Some(process_info.id));
                    }
                });
            }
        });
}
