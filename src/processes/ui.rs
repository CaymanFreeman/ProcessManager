use crate::app;
use crate::processes::data;
use std::ops::RangeInclusive;

const HEADER_TEXT_SIZE: f32 = 12.0;
const HEADER_HEIGHT: f32 = 25.0;
const ROW_HEIGHT: f32 = 18.0;

const CONTROL_PANEL_MIN_HEIGHT: f32 = 40.0;
const CONTROL_BUTTON_TEXT_SIZE: f32 = 16.0;
const CONTROL_BUTTON_SIZE: [f32; 2] = [50.0, 25.0];

const COLUMN_WIDTH_RANGE: RangeInclusive<f32> = 75.0..=500.0;
const BLANK_PROCESS_PATH: &str = "";
const BLANK_PROCESS_NAME: &str = "";

pub fn update(app: &mut app::App, ctx: &egui::Context) {
    egui::TopBottomPanel::top("options_bar").show(ctx, |ui| {
        update_options_panel(app, ui);
    });
    egui::TopBottomPanel::bottom("control_bar")
        .min_height(CONTROL_PANEL_MIN_HEIGHT)
        .show(ctx, |ui| {
            update_control_bar(app, ctx, ui);
        });
    egui::CentralPanel::default().show(ctx, |ui| {
        update_table(app, ui);
    });
}

fn update_options_panel(app: &mut app::App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut app.process_filter).hint_text("Filter processes"));
        ui.checkbox(&mut app.show_thread_processes, "Threads");
    });
}

fn control_button(text: impl Into<String>, ui: &mut egui::Ui) -> egui::Response {
    ui.add(
        egui::Button::new(
            egui::RichText::new(text).font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
        )
        .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
    )
}

#[expect(clippy::collapsible_if)]
fn update_control_bar(app: &app::App, ctx: &egui::Context, ui: &mut egui::Ui) {
    if let (Some(pid), Ok(system)) = (app.selected_pid, app.system.lock()) {
        if let Some(process) = system.processes().get(&sysinfo::Pid::from_u32(pid)) {
            ui.horizontal_centered(|ui| {
                if control_button("Terminate", ui).clicked() {
                    process.kill_with(sysinfo::Signal::Term);
                }

                if control_button("Kill", ui).clicked() {
                    process.kill_with(sysinfo::Signal::Kill);
                }

                if control_button("Copy Path", ui).clicked() {
                    ctx.copy_text(
                        data::extract_path(process)
                            .unwrap_or(BLANK_PROCESS_PATH)
                            .to_owned(),
                    );
                }

                if control_button("Copy Name", ui).clicked() {
                    ctx.copy_text(
                        data::extract_name(process)
                            .unwrap_or(BLANK_PROCESS_NAME)
                            .to_owned(),
                    );
                }

                if control_button("Copy PID", ui).clicked() {
                    ctx.copy_text(process.pid().to_string());
                }
            });
        }
    }
}

fn header_label(text: impl Into<String>, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(text).font(egui::FontId::proportional(HEADER_TEXT_SIZE)));
}

fn cell_label(text: impl Into<egui::WidgetText>, ui: &mut egui::Ui) {
    ui.style_mut().interaction.selectable_labels = false;
    ui.add(egui::Label::new(text).truncate());
}

fn update_table(app: &mut app::App, ui: &mut egui::Ui) {
    let processes_info = data::prepare_processes(app);
    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .columns(
            egui_extras::Column::auto_with_initial_suggestion(*COLUMN_WIDTH_RANGE.start())
                .range(COLUMN_WIDTH_RANGE)
                .resizable(true),
            9,
        )
        .header(HEADER_HEIGHT, |mut header_row| {
            header_row.col(|ui| header_label("Name", ui));
            header_row.col(|ui| header_label("ID", ui));
            header_row.col(|ui| header_label("User", ui));
            header_row.col(|ui| header_label("Memory", ui));
            header_row.col(|ui| header_label("CPU", ui));
            header_row.col(|ui| header_label("Disk Read", ui));
            header_row.col(|ui| header_label("Disk Write", ui));
            header_row.col(|ui| header_label("Path", ui));
            header_row.col(|ui| header_label("Status", ui));
        })
        .body(|mut body_rows| {
            for process_info in &processes_info {
                body_rows.row(ROW_HEIGHT, |mut row| {
                    row.set_selected(app.selected_pid == Some(process_info.id));

                    row.col(|ui| cell_label(&process_info.name, ui));
                    row.col(|ui| cell_label(process_info.id.to_string(), ui));
                    row.col(|ui| cell_label(&process_info.user, ui));
                    row.col(|ui| cell_label(&process_info.memory, ui));
                    row.col(|ui| cell_label(&process_info.cpu, ui));
                    row.col(|ui| cell_label(&process_info.disk_read, ui));
                    row.col(|ui| cell_label(&process_info.disk_write, ui));
                    row.col(|ui| cell_label(&process_info.path, ui));
                    row.col(|ui| cell_label(&process_info.status, ui));

                    let response = row.response();
                    if response.hovered() && response.ctx.input(|i| i.pointer.primary_clicked()) {
                        app.selected_pid = Some(process_info.id);
                    }
                });
            }
        });
}
