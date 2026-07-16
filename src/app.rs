//! The eframe application: owns UI state, drains worker messages, and
//! re-rasterizes the radar texture only when scan/product/tilt changes.

use crate::borders::BorderMessage;
use crate::data::{SITE, WorkerMessage};
use crate::model::{Product, ScanData};
use crate::scope;
use chrono::{DateTime, Utc};
use egui::{Color32, TextureHandle, TextureOptions, Ui};
use std::sync::mpsc::Receiver;

pub struct RadarApp {
    rx: Receiver<WorkerMessage>,
    border_rx: Receiver<BorderMessage>,
    scan: Option<ScanData>,
    product: Product,
    tilt_index: usize,
    status: String,
    texture: Option<TextureHandle>,
    texture_key: Option<(DateTime<Utc>, Product, usize)>,
    borders: Vec<crate::borders::Ring>,
}

impl RadarApp {
    pub fn new(rx: Receiver<WorkerMessage>, border_rx: Receiver<BorderMessage>) -> Self {
        Self {
            rx,
            border_rx,
            scan: None,
            product: Product::Reflectivity,
            tilt_index: 0,
            status: format!("Starting up — fetching latest {SITE} volume…"),
            texture: None,
            texture_key: None,
            borders: Vec::new(),
        }
    }

    fn drain_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                WorkerMessage::NewScan(scan) => {
                    self.status = format!(
                        "Scan {} UTC ({} local)",
                        scan.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        scan.timestamp
                            .with_timezone(&chrono::Local)
                            .format("%H:%M:%S")
                    );
                    self.scan = Some(*scan);
                }
                WorkerMessage::Status(text) => self.status = text,
                WorkerMessage::Error(text) => self.status = format!("Error: {text}"),
            }
        }
        while let Ok(message) = self.border_rx.try_recv() {
            match message {
                BorderMessage::Loaded(rings) => self.borders = rings,
                BorderMessage::Error(text) => {
                    self.status = format!("State borders unavailable: {text}");
                }
            }
        }
    }

    /// Clamp the tilt index to the sweeps available for the current product
    /// (velocity may have fewer tilts than reflectivity).
    fn clamped_tilt(&self) -> usize {
        let count = self
            .scan
            .as_ref()
            .map_or(0, |s| s.sweeps(self.product).len());
        self.tilt_index.min(count.saturating_sub(1))
    }

    fn ensure_texture(&mut self, ctx: &egui::Context) {
        let Some(scan) = &self.scan else { return };
        let sweeps = scan.sweeps(self.product);
        if sweeps.is_empty() {
            self.texture = None;
            self.texture_key = None;
            return;
        }

        let tilt = self.clamped_tilt();
        let key = (scan.timestamp, self.product, tilt);
        if self.texture_key == Some(key) && self.texture.is_some() {
            return;
        }

        let image = scope::rasterize(
            &sweeps[tilt],
            self.product,
            scope::RASTER_SIZE_PX,
            scope::MAX_RANGE_KM,
        );
        self.texture = Some(ctx.load_texture("radar-sweep", image, TextureOptions::LINEAR));
        self.texture_key = Some(key);
    }
}

impl eframe::App for RadarApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.drain_messages();
        self.ensure_texture(&ctx);

        egui::Panel::top("controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{SITE} — Macon, GA")).strong());
                ui.separator();
                ui.selectable_value(
                    &mut self.product,
                    Product::Reflectivity,
                    Product::Reflectivity.label(),
                );
                ui.selectable_value(
                    &mut self.product,
                    Product::Velocity,
                    Product::Velocity.label(),
                );
                ui.separator();

                if let Some(scan) = &self.scan {
                    let sweeps = scan.sweeps(self.product);
                    if !sweeps.is_empty() {
                        let current = self.clamped_tilt();
                        egui::ComboBox::from_label("Tilt")
                            .selected_text(format!("{:.1}°", sweeps[current].elevation_deg))
                            .show_ui(ui, |ui| {
                                for (i, sweep) in sweeps.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.tilt_index,
                                        i,
                                        format!("{:.1}°", sweep.elevation_deg),
                                    );
                                }
                            });
                    }
                }
            });
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(6, 9, 14)))
            .show(ui, |ui| {
                scope::draw_scope(ui, self.texture.as_ref(), self.scan.as_ref(), self.product);
            });
    }
}
