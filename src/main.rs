fn main() -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel();
    let (border_tx, border_rx) = std::sync::mpsc::channel();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 960.0])
            .with_title("rustywx — KJGX radarscope (Macon, GA)"),
        ..Default::default()
    };

    eframe::run_native(
        "rustywx",
        options,
        Box::new(move |cc| {
            rustywx::data::spawn_worker(tx, cc.egui_ctx.clone());
            rustywx::borders::spawn_border_loader(border_tx, cc.egui_ctx.clone());
            Ok(Box::new(rustywx::app::RadarApp::new(rx, border_rx)))
        }),
    )
}
