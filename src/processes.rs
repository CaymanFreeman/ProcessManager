use crate::app;
use egui_extras::TableBody;
use std::cmp::Ordering;
use std::ops::RangeInclusive;

const HEADER_TEXT_SIZE: f32 = 12.0;
const HEADER_HEIGHT: f32 = 25.0;
const ROW_HEIGHT: f32 = 18.0;

const CONTROL_PANEL_MIN_HEIGHT: f32 = 40.0;
const CONTROL_BUTTON_TEXT_SIZE: f32 = 16.0;
const CONTROL_BUTTON_SIZE: [f32; 2] = [50.0, 25.0];

const DASH: &str = "-";

struct Column {
    name: &'static str,
    width_range: RangeInclusive<f32>,
}

const COLUMNS: &[Column] = &[
    Column {
        name: "Name",
        width_range: 120.0..=500.0,
    },
    Column {
        name: "ID",
        width_range: 60.0..=200.0,
    },
    Column {
        name: "User",
        width_range: 120.0..=250.0,
    },
    Column {
        name: "Memory",
        width_range: 60.0..=200.0,
    },
    Column {
        name: "CPU",
        width_range: 60.0..=200.0,
    },
    Column {
        name: "Disk Read",
        width_range: 70.0..=200.0,
    },
    Column {
        name: "Disk Write",
        width_range: 70.0..=200.0,
    },
    Column {
        name: "Path",
        width_range: 70.0..=500.0,
    },
    Column {
        name: "Status",
        width_range: 60.0..=200.0,
    },
];

pub fn update(app: &mut app::App, ctx: &egui::Context) {
    update_options_panel(app, ctx);
    update_process_control_bar(app, ctx);
    egui::CentralPanel::default().show(ctx, |ui| {
        update_processes_table(app, ui);
    });
}

fn update_processes_table(app: &mut app::App, ui: &mut egui::Ui) {
    let users = sysinfo::Users::new_with_refreshed_list();
    let mut table = egui_extras::TableBuilder::new(ui).striped(true);
    for col in COLUMNS {
        table = table.column(
            egui_extras::Column::initial(*col.width_range.start())
                .range(col.width_range.clone())
                .resizable(true),
        );
    }

    table
        .header(HEADER_HEIGHT, table_header())
        .body(table_body(app, &users));
}

fn table_body(app: &mut app::App, users: &sysinfo::Users) -> impl FnOnce(TableBody<'_>) {
    move |mut body| {
        let system = app.system.lock().expect("Failed to acquire system lock");
        let mut processes: Vec<&sysinfo::Process> = system.processes().values().collect();
        if !app.process_filter.is_empty() {
            processes.retain(|process| {
                process
                    .name()
                    .to_str()
                    .map(|name| {
                        name.to_lowercase()
                            .contains(&app.process_filter.to_lowercase())
                    })
                    .unwrap_or(false)
            });
        }

        if !app.show_thread_processes {
            processes.retain(|process| process.thread_kind().is_none());
        }

        processes.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(Ordering::Equal)
        });

        for process in processes {
            body.row(ROW_HEIGHT, |mut row| {
                row.set_selected(app.selected_pid == Some(process.pid()));

                // Name
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(egui::Label::new(extract_process_name(process, true)).truncate());
                });

                // ID
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(egui::Label::new(process.pid().to_string()).truncate());
                });

                // User
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(egui::Label::new(get_user_name(process, users, true)).truncate());
                });

                // Memory
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(bytesize::ByteSize(process.memory()).to_string())
                            .truncate(),
                    );
                });

                // CPU
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(format!(
                            "{:.2}%",
                            process.cpu_usage() / system.cpus().len() as f32
                        ))
                        .truncate(),
                    );
                });

                // Disk Read
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(
                            bytesize::ByteSize(process.disk_usage().read_bytes).to_string(),
                        )
                        .truncate(),
                    );
                });

                // Disk Write
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(
                        egui::Label::new(
                            bytesize::ByteSize(process.disk_usage().written_bytes).to_string(),
                        )
                        .truncate(),
                    );
                });

                // Path
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(egui::Label::new(extract_process_path(process, true)).truncate());
                });

                // Status
                row.col(|ui| {
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.add(egui::Label::new(process.status().to_string()).truncate());
                });

                let response = row.response();
                if response.hovered() && response.ctx.input(|i| i.pointer.primary_clicked()) {
                    app.selected_pid = Some(process.pid());
                }
            });
        }
    }
}

fn table_header() -> impl FnOnce(egui_extras::TableRow<'_, '_>) {
    |mut header| {
        for col in COLUMNS {
            header.col(|ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new(col.name)
                            .font(egui::FontId::proportional(HEADER_TEXT_SIZE)),
                    );
                });
            });
        }
    }
}

fn update_options_panel(app: &mut app::App, ctx: &egui::Context) {
    egui::TopBottomPanel::top("options_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.process_filter).hint_text("Filter processes"),
            );
            ui.checkbox(&mut app.show_thread_processes, "Threads");
        });
    });
}

fn update_process_control_bar(app: &mut app::App, ctx: &egui::Context) {
    if let Some(pid) = app.selected_pid {
        let system = app.system.lock().expect("Failed to acquire system lock");
        if let Some(process) = system.processes().get(&pid) {
            egui::TopBottomPanel::bottom("control_bar")
                .min_height(CONTROL_PANEL_MIN_HEIGHT)
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Terminate")
                                        .font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
                                )
                                .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
                            )
                            .clicked()
                        {
                            process.kill_with(sysinfo::Signal::Term);
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Kill")
                                        .font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
                                )
                                .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
                            )
                            .clicked()
                        {
                            process.kill_with(sysinfo::Signal::Kill);
                        }

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Copy Path")
                                        .font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
                                )
                                .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
                            )
                            .clicked()
                        {
                            ctx.copy_text(extract_process_path(process, false).to_owned());
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Copy Name")
                                        .font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
                                )
                                .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
                            )
                            .clicked()
                        {
                            ctx.copy_text(extract_process_name(process, false).to_owned());
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Copy PID")
                                        .font(egui::FontId::proportional(CONTROL_BUTTON_TEXT_SIZE)),
                                )
                                .min_size(egui::Vec2::from(CONTROL_BUTTON_SIZE)),
                            )
                            .clicked()
                        {
                            ctx.copy_text(process.pid().to_string());
                        }
                    });
                });
        } else {
            app.selected_pid = None;
        }
    }
}

fn extract_process_path(process: &sysinfo::Process, dash_if_none: bool) -> &str {
    process
        .exe()
        .and_then(|path| path.to_str())
        .unwrap_or(if dash_if_none { DASH } else { "" })
}

fn extract_process_name(process: &sysinfo::Process, dash_if_none: bool) -> &str {
    process
        .name()
        .to_str()
        .unwrap_or(if dash_if_none { DASH } else { "" })
}

fn get_user_name<'a>(
    process: &sysinfo::Process,
    users: &'a sysinfo::Users,
    dash_if_none: bool,
) -> &'a str {
    process
        .user_id()
        .and_then(|uid| users.get_user_by_id(uid))
        .map(|user| user.name())
        .unwrap_or_else(|| if dash_if_none { DASH } else { "" })
}
