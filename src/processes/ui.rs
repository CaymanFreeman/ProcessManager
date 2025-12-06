use crate::app;
use crate::processes::data;
use crate::processes::data::{SortCategory, SortDirection, SortMethod};
use std::ops::RangeInclusive;

const HEADER_TEXT_SIZE: f32 = 12.0;
const HEADER_HEIGHT: f32 = 25.0;
const ROW_HEIGHT: f32 = 18.0;

const CONTROL_PANEL_MIN_HEIGHT: f32 = 40.0;
const CONTROL_BUTTON_TEXT_SIZE: f32 = 16.0;
const CONTROL_BUTTON_SIZE: [f32; 2] = [50.0, 25.0];

const COLUMN_WIDTH_RANGE: RangeInclusive<f32> = 90.0..=500.0;
const BLANK_PROCESS_PATH: &str = "";
const BLANK_PROCESS_NAME: &str = "";

const ASCENDING_SYMBOL: &str = "⏶";
const DESCENDING_SYMBOL: &str = "⏷";

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

fn header_name_label(text: &str, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new(text).font(egui::FontId::proportional(HEADER_TEXT_SIZE)));
}

fn header_sort_label(sort_direction: &SortDirection, ui: &mut egui::Ui) {
    match sort_direction {
        SortDirection::Ascending => {
            ui.label(
                egui::RichText::new(ASCENDING_SYMBOL)
                    .font(egui::FontId::proportional(HEADER_TEXT_SIZE)),
            );
        }
        SortDirection::Descending => {
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
    header_category: Option<SortCategory>,
    current_sort_method: &mut SortMethod,
    ui: &mut egui::Ui,
) {
    ui.style_mut().interaction.selectable_labels = false;

    if let Some(sort_category) = header_category {
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

        if response_primary_clicked(&ui.response()) {
            if current_sort_method.category == sort_category {
                current_sort_method.toggle_direction();
            } else {
                current_sort_method.category = sort_category;
                current_sort_method.direction = SortDirection::Ascending;
            }
        }
    } else {
        ui.horizontal_centered(|ui| {
            header_name_label(text, ui);
        });
    }
}

fn body_cell(text: &str, ui: &mut egui::Ui) {
    ui.style_mut().interaction.selectable_labels = false;
    ui.add(egui::Label::new(text).truncate());
}

fn format_bytes(bytes: u64) -> String {
    bytesize::ByteSize(bytes).to_string()
}

fn update_table(app: &mut app::App, ui: &mut egui::Ui) {
    let processes_info = data::prepare_processes(app);
    egui_extras::TableBuilder::new(ui)
        .striped(true)
        .columns(
            egui_extras::Column::remainder()
                .range(COLUMN_WIDTH_RANGE)
                .resizable(true),
            9,
        )
        .header(HEADER_HEIGHT, |mut header_row| {
            let sort_method = &mut app.sort_method;
            header_row.col(|ui| header_cell("Name", Some(SortCategory::Name), sort_method, ui));
            header_row.col(|ui| header_cell("ID", Some(SortCategory::Id), sort_method, ui));
            header_row.col(|ui| header_cell("User", Some(SortCategory::User), sort_method, ui));
            header_row.col(|ui| header_cell("Memory", Some(SortCategory::Memory), sort_method, ui));
            header_row.col(|ui| header_cell("CPU", Some(SortCategory::Cpu), sort_method, ui));
            header_row
                .col(|ui| header_cell("Disk Read", Some(SortCategory::DiskRead), sort_method, ui));
            header_row.col(|ui| {
                header_cell("Disk Write", Some(SortCategory::DiskWrite), sort_method, ui);
            });
            header_row.col(|ui| header_cell("Path", None, sort_method, ui));
            header_row.col(|ui| header_cell("Status", Some(SortCategory::Status), sort_method, ui));
        })
        .body(|mut body_rows| {
            for process_info in &processes_info {
                body_rows.row(ROW_HEIGHT, |mut row| {
                    row.set_selected(app.selected_pid == Some(process_info.id));

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
                        app.selected_pid = Some(process_info.id);
                    }
                });
            }
        });
}
