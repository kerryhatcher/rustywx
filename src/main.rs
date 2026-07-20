fn main() -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel();
    let (border_tx, border_rx) = std::sync::mpsc::channel();
    let (alert_tx, alert_rx) = std::sync::mpsc::channel();
    let (nhc_tx, nhc_rx) = std::sync::mpsc::channel();
    let (nhc_refresh_tx, nhc_refresh_rx) = std::sync::mpsc::channel();
    let (site_tx, site_rx) = std::sync::mpsc::channel();

    let default_site = rustywx::geo::RADAR_SITES[0];
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 960.0])
            .with_title(format!(
                "rustywx — {} radarscope ({})",
                default_site.id, default_site.name
            )),
        ..Default::default()
    };

    eframe::run_native(
        "rustywx",
        options,
        Box::new(move |cc| {
            rustywx::data::spawn_worker(
                tx,
                cc.egui_ctx.clone(),
                default_site.id.to_string(),
                site_rx,
            );
            rustywx::borders::spawn_border_loader(border_tx, cc.egui_ctx.clone());
            rustywx::alerts::spawn_alerts_worker(alert_tx, cc.egui_ctx.clone());
            rustywx::nhc::spawn_nhc_worker(nhc_tx, cc.egui_ctx.clone(), nhc_refresh_rx);
            Ok(Box::new(rustywx::app::RadarApp::new(
                rx,
                border_rx,
                alert_rx,
                nhc_rx,
                nhc_refresh_tx,
                site_tx,
            )))
        }),
    )
}
