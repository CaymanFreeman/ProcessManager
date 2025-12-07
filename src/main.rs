#![warn(clippy::all, rust_2018_idioms)]

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_clamp_size_to_monitor_size(true)
            .with_inner_size([1025.0, 720.0])
            .with_app_id("process_manager")
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Process Manager",
        native_options,
        Box::new(|cc| Ok(Box::new(process_manager::App::new(cc)))),
    )
}
