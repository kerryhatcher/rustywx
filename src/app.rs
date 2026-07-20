//! The eframe application: owns UI state, drains worker messages, and
//! re-rasterizes the radar texture only when scan/product/tilt changes.

use crate::alerts::{Alert, AlertMessage};
use crate::borders::BorderMessage;
use crate::data::WorkerMessage;
use crate::geo::RadarSite;
use crate::model::{Product, ScanData};
use crate::nhc::{NhcBundle, Nhcmessage};
use crate::scope;
use chrono::{DateTime, Utc};
use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Ui, Vec2};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

pub struct RadarApp {
    rx: Receiver<WorkerMessage>,
    border_rx: Receiver<BorderMessage>,
    alert_rx: Receiver<AlertMessage>,
    nhc_rx: Receiver<Nhcmessage>,
    nhc_refresh_tx: std::sync::mpsc::Sender<()>,
    scan: Option<ScanData>,
    product: Product,
    tilt_index: usize,
    site_index: usize,
    site_tx: Sender<String>,
    status: String,
    texture: Option<TextureHandle>,
    texture_key: Option<(DateTime<Utc>, Product, usize)>,
    borders: Vec<crate::borders::Ring>,
    alerts: Vec<Alert>,
    /// Full NHC data bundle (metadata, GIS, text, images).
    nhc_bundle: Option<NhcBundle>,
    /// Decoded image textures keyed by "storm_id:product_title".
    nhc_image_textures: HashMap<String, TextureHandle>,
    /// Currently selected storm ID for the detail panel.
    nhc_selected_storm: Option<String>,
    /// Whether the NHC detail panel is visible.
    nhc_show_panel: bool,
    /// Overlay toggle state.
    show_wind_probs_34kt: bool,
    show_wind_probs_50kt: bool,
    show_wind_probs_64kt: bool,
    show_earliest_arrival: bool,
    show_most_likely_arrival: bool,
    /// Pan offset in kilometres from the radar site.
    pan_km: Vec2,
    /// Zoom level: 1.0 = default 230 km range, higher = zoomed in.
    zoom: f32,
}

impl RadarApp {
    pub fn new(
        rx: Receiver<WorkerMessage>,
        border_rx: Receiver<BorderMessage>,
        alert_rx: Receiver<AlertMessage>,
        nhc_rx: Receiver<Nhcmessage>,
        nhc_refresh_tx: std::sync::mpsc::Sender<()>,
        site_tx: Sender<String>,
    ) -> Self {
        let saved = crate::store::load_state().unwrap_or(None);
        let site_index = saved
            .as_ref()
            .and_then(|s| {
                crate::geo::RADAR_SITES
                    .iter()
                    .position(|r| r.id == s.site_id)
            })
            .unwrap_or(0);
        let product = saved
            .as_ref()
            .map(|s| match s.product.as_str() {
                "Velocity" => Product::Velocity,
                _ => Product::Reflectivity,
            })
            .unwrap_or(Product::Reflectivity);
        let tilt_index = saved.as_ref().map_or(0, |s| s.tilt_index);

        let default_site = crate::geo::RADAR_SITES[site_index];
        if site_index != 0 {
            let _ = site_tx.send(default_site.id.to_string());
        }

        let cached_scan = crate::cache::load_radar(default_site.id);
        let cached_alerts = crate::cache::load_alerts().unwrap_or_default();
        let cached_nhc = crate::cache::load_nhc();
        let status = if cached_scan.is_some() {
            "Loaded from cache — checking for updates…".to_string()
        } else {
            format!("Starting up — fetching latest {} volume…", default_site.id)
        };

        Self {
            rx,
            border_rx,
            alert_rx,
            nhc_rx,
            nhc_refresh_tx,
            scan: cached_scan,
            product,
            tilt_index,
            site_index,
            site_tx,
            status,
            texture: None,
            texture_key: None,
            borders: Vec::new(),
            alerts: cached_alerts,
            nhc_bundle: cached_nhc,
            nhc_image_textures: HashMap::new(),
            nhc_selected_storm: None,
            nhc_show_panel: false,
            show_wind_probs_34kt: false,
            show_wind_probs_50kt: false,
            show_wind_probs_64kt: false,
            show_earliest_arrival: false,
            show_most_likely_arrival: false,
            pan_km: Vec2::ZERO,
            zoom: 1.0,
        }
    }

    fn current_site(&self) -> &'static RadarSite {
        &crate::geo::RADAR_SITES[self.site_index]
    }

    fn save_current_state(&self) {
        let state = crate::store::AppState {
            site_id: self.current_site().id.to_string(),
            product: self.product.label().to_string(),
            tilt_index: self.tilt_index,
        };
        let _ = crate::store::save_state(&state);
    }

    fn drain_messages(&mut self, ctx: &egui::Context) {
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
        while let Ok(message) = self.alert_rx.try_recv() {
            match message {
                AlertMessage::Loaded(alerts) => self.alerts = alerts,
                AlertMessage::Error(text) => {
                    self.status = format!("NWS alerts unavailable: {text}");
                }
            }
        }
        while let Ok(message) = self.nhc_rx.try_recv() {
            match message {
                Nhcmessage::Loaded(bundle) => {
                    // Decode newly arrived images into textures.
                    for (storm_id, images) in &bundle.image_products {
                        for img in images {
                            if let Some(ref data) = img.data
                                && let Ok(rgba) = decode_image_to_rgba(data)
                            {
                                let key = format!("{storm_id}:{}", img.title);
                                let texture = ctx.load_texture(&key, rgba, TextureOptions::LINEAR);
                                self.nhc_image_textures.insert(key, texture);
                            }
                        }
                    }
                    // Auto-select first storm if nothing selected.
                    if self.nhc_selected_storm.is_none() && !bundle.metas.is_empty() {
                        self.nhc_selected_storm = Some(bundle.metas[0].id.clone());
                        self.nhc_show_panel = true;
                    }
                    self.nhc_bundle = Some(bundle);
                }
                Nhcmessage::Error(text) => {
                    self.status = format!("NHC data unavailable: {text}");
                }
            }
        }
    }

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

    /// Build the NHC detail panel contents.
    fn show_nhc_panel(&mut self, ui: &mut Ui) {
        let Some(ref bundle) = self.nhc_bundle else {
            ui.label("No NHC data available.");
            return;
        };

        if bundle.metas.is_empty() {
            ui.label("No active tropical cyclones.");
            return;
        }

        // Storm selector dropdown.
        ui.horizontal(|ui| {
            ui.label("Storm:");
            let selected_name = self
                .nhc_selected_storm
                .as_deref()
                .and_then(|id| {
                    bundle
                        .metas
                        .iter()
                        .find(|m| m.id == id)
                        .map(|m| format!("{} — {} ({})", m.name, m.classification, m.id))
                })
                .unwrap_or_else(|| "Select…".to_string());

            egui::ComboBox::from_id_salt("nhc-storm-selector")
                .selected_text(&selected_name)
                .show_ui(ui, |ui| {
                    for meta in &bundle.metas {
                        let label = format!(
                            "{} — {} ({}kt, {}mb)",
                            meta.name, meta.classification, meta.intensity_kt, meta.pressure_mb
                        );
                        if ui
                            .selectable_value(
                                &mut self.nhc_selected_storm,
                                Some(meta.id.clone()),
                                label,
                            )
                            .changed()
                        {
                            // Clear old image textures for this storm to
                            // force reload on next cycle.
                        }
                    }
                });
        });

        let Some(ref storm_id) = self.nhc_selected_storm else {
            return;
        };
        let Some(meta) = bundle.metas.iter().find(|m| &m.id == storm_id) else {
            ui.label("Storm not found.");
            return;
        };

        egui::ScrollArea::vertical()
            .id_salt("nhc-scroll")
            .show(ui, |ui| {
                // ── Storm info card ─────────────────────────────────
                ui.heading(format!("{} {}", meta.name, meta.classification));
                ui.add_space(4.0);

                egui::Grid::new("nhc-stats")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        let bold = egui::RichText::new;
                        ui.label(bold("Intensity:"));
                        ui.label(format!("{} kt", meta.intensity_kt));
                        ui.end_row();

                        ui.label(bold("Pressure:"));
                        ui.label(format!("{} mb", meta.pressure_mb));
                        ui.end_row();

                        ui.label(bold("Position:"));
                        ui.label(format!("{:.1}°N, {:.1}°W", meta.lat, -meta.lon));
                        ui.end_row();

                        if let (Some(dir), Some(spd)) =
                            (meta.movement_dir_deg, meta.movement_speed_kt)
                        {
                            ui.label(bold("Movement:"));
                            ui.label(format!("{}° at {} kt", dir, spd));
                            ui.end_row();
                        }

                        ui.label(bold("Advisory:"));
                        ui.label(&meta.advisory_num);
                        ui.end_row();

                        ui.label(bold("Updated:"));
                        ui.label(&meta.last_update);
                        ui.end_row();
                    });

                ui.add_space(8.0);

                // ── Graphics page link ──────────────────────────────
                if !meta.graphics_url.is_empty() {
                    if ui.button("🌐 Open NHC Graphics Page").clicked() {
                        let _ = webbrowser::open(&meta.graphics_url);
                    }
                    ui.add_space(8.0);
                }

                // ── Image products ──────────────────────────────────
                ui.separator();
                ui.label(egui::RichText::new("Graphics Products").strong().size(14.0));
                ui.add_space(4.0);

                if let Some((_, images)) =
                    bundle.image_products.iter().find(|(id, _)| id == storm_id)
                {
                    for img in images {
                        let key = format!("{storm_id}:{}", img.title);
                        let has_texture = self.nhc_image_textures.contains_key(&key);

                        ui.horizontal(|ui| {
                            if has_texture {
                                // Show thumbnail.
                                if let Some(tex) = self.nhc_image_textures.get(&key) {
                                    let available = ui.available_width().min(200.0);
                                    let aspect = tex.size_vec2().x / tex.size_vec2().y.max(1.0);
                                    let height = available / aspect;
                                    ui.image(egui::ImageSource::Texture(
                                        egui::load::SizedTexture::new(
                                            tex.id(),
                                            [available, height],
                                        ),
                                    ));
                                }
                            } else {
                                // Placeholder.
                                ui.add_sized(
                                    [200.0, 120.0],
                                    egui::Label::new(
                                        egui::RichText::new("Loading…").color(Color32::GRAY),
                                    ),
                                );
                            }

                            ui.vertical(|ui| {
                                ui.label(egui::RichText::new(&img.title).strong());
                                if ui.button("🔗 Open").clicked() {
                                    let _ = webbrowser::open(&img.url);
                                }
                            });
                        });
                        ui.add_space(4.0);
                    }
                }

                ui.add_space(8.0);

                // ── Overlay toggles ────────────────────────────────
                ui.separator();
                ui.label(egui::RichText::new("Map Overlays").strong().size(14.0));
                ui.add_space(4.0);

                let has_34kt = bundle.wind_probs_34kt.is_empty();
                let has_50kt = bundle.wind_probs_50kt.is_empty();
                let has_64kt = bundle.wind_probs_64kt.is_empty();
                let has_earliest = bundle.earliest_arrival.is_empty();
                let has_most_likely = bundle.most_likely_arrival.is_empty();

                ui.add_enabled_ui(!has_34kt, |ui| {
                    ui.checkbox(&mut self.show_wind_probs_34kt, "34kt Wind Probability");
                });
                ui.add_enabled_ui(!has_50kt, |ui| {
                    ui.checkbox(&mut self.show_wind_probs_50kt, "50kt Wind Probability");
                });
                ui.add_enabled_ui(!has_64kt, |ui| {
                    ui.checkbox(&mut self.show_wind_probs_64kt, "64kt Wind Probability");
                });
                ui.add_enabled_ui(!has_earliest, |ui| {
                    ui.checkbox(&mut self.show_earliest_arrival, "Earliest Arrival (34kt)");
                });
                ui.add_enabled_ui(!has_most_likely, |ui| {
                    ui.checkbox(
                        &mut self.show_most_likely_arrival,
                        "Most Likely Arrival (34kt)",
                    );
                });

                ui.add_space(8.0);

                // ── Text products ───────────────────────────────────
                ui.separator();
                ui.label(egui::RichText::new("Text Products").strong().size(14.0));
                ui.add_space(4.0);

                if let Some((_, texts)) = bundle.text_products.iter().find(|(id, _)| id == storm_id)
                {
                    for product in texts {
                        ui.collapsing(&product.title, |ui| {
                            // Show a link to open in browser.
                            if ui.button("🔗 Open in browser").clicked() {
                                let _ = webbrowser::open(&product.url);
                            }
                            ui.add_space(4.0);

                            // Show the text content in a scrollable area.
                            egui::ScrollArea::vertical()
                                .id_salt(format!("nhc-text-{}", product.title))
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    ui.label(
                                        egui::RichText::new(&product.content)
                                            .monospace()
                                            .size(11.0),
                                    );
                                });
                        });
                    }
                }
            });
    }
}

/// Decode raw image bytes (PNG or JPEG) into an egui ColorImage.
fn decode_image_to_rgba(data: &[u8]) -> Result<ColorImage, String> {
    let img = image::load_from_memory(data).map_err(|e| format!("decode: {e}"))?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        rgba.as_raw(),
    ))
}

impl eframe::App for RadarApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        self.drain_messages(&ctx);
        self.ensure_texture(&ctx);

        // ── Top controls bar ───────────────────────────────────────
        egui::Panel::top("controls").show(ui, |ui| {
            ui.horizontal(|ui| {
                let site = self.current_site();
                ui.label(egui::RichText::new(format!("{} — {}", site.id, site.name)).strong());
                ui.separator();

                egui::ComboBox::from_label("Site")
                    .selected_text(site.id)
                    .show_ui(ui, |ui| {
                        for (i, s) in crate::geo::RADAR_SITES.iter().enumerate() {
                            if ui
                                .selectable_value(
                                    &mut self.site_index,
                                    i,
                                    format!("{} — {}", s.id, s.name),
                                )
                                .changed()
                            {
                                let _ = self.site_tx.send(s.id.to_string());
                                self.scan = None;
                                self.texture = None;
                                self.texture_key = None;
                                self.tilt_index = 0;
                                self.status = format!("Switching to {} — fetching data…", s.id);
                                self.save_current_state();
                                let _ = self.nhc_refresh_tx.send(());
                            }
                        }
                    });
                ui.separator();

                if ui
                    .selectable_value(
                        &mut self.product,
                        Product::Reflectivity,
                        Product::Reflectivity.label(),
                    )
                    .changed()
                {
                    self.save_current_state();
                }
                if ui
                    .selectable_value(
                        &mut self.product,
                        Product::Velocity,
                        Product::Velocity.label(),
                    )
                    .changed()
                {
                    self.save_current_state();
                }
                ui.separator();

                if let Some(scan) = &self.scan {
                    let sweeps = scan.sweeps(self.product);
                    if !sweeps.is_empty() {
                        let current = self.clamped_tilt();
                        let mut tilt_changed = false;
                        egui::ComboBox::from_label("Tilt")
                            .selected_text(format!("{:.1}°", sweeps[current].elevation_deg))
                            .show_ui(ui, |ui| {
                                for (i, sweep) in sweeps.iter().enumerate() {
                                    if ui
                                        .selectable_value(
                                            &mut self.tilt_index,
                                            i,
                                            format!("{:.1}°", sweep.elevation_deg),
                                        )
                                        .changed()
                                    {
                                        tilt_changed = true;
                                    }
                                }
                            });
                        if tilt_changed {
                            self.save_current_state();
                        }
                    }
                }

                ui.separator();

                // NHC panel toggle button.
                let has_storms = self
                    .nhc_bundle
                    .as_ref()
                    .is_some_and(|b| !b.metas.is_empty());
                let toggle_label = if self.nhc_show_panel {
                    "🟢 NHC"
                } else if has_storms {
                    "🔴 NHC"
                } else {
                    "⚪ NHC"
                };
                if ui.button(toggle_label).clicked() {
                    self.nhc_show_panel = !self.nhc_show_panel;
                }
                if has_storms {
                    ui.label(format!(
                        "{} active",
                        self.nhc_bundle.as_ref().unwrap().metas.len()
                    ));
                }
            });
        });

        // ── Bottom status bar ──────────────────────────────────────
        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(&self.status);
        });

        // ── Main layout: scope + optional NHC panel ────────────────

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(Color32::from_rgb(6, 9, 14)))
            .show(ui, |ui| {
                // Capture the available rect before drawing the scope,
                // so we can position the NHC panel overlay correctly.
                let central_rect = ui.available_rect_before_wrap();

                self.show_scope(ui);

                if self.nhc_show_panel {
                    let panel_width = 340.0;
                    egui::Area::new("nhc-panel".into())
                        .fixed_pos(egui::pos2(
                            central_rect.right() - panel_width - 8.0,
                            central_rect.top() + 8.0,
                        ))
                        .show(ui.ctx(), |ui| {
                            egui::Frame::new()
                                .fill(Color32::from_rgb(18, 22, 30))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.set_max_width(panel_width);
                                    ui.set_max_height(central_rect.height() - 16.0);
                                    egui::ScrollArea::vertical()
                                        .id_salt("nhc-panel-scroll")
                                        .show(ui, |ui| {
                                            self.show_nhc_panel(ui);
                                        });
                                });
                        });
                }
            });
    }
}

impl RadarApp {
    /// Render the radar scope with overlays.
    fn show_scope(&mut self, ui: &mut Ui) {
        // Mouse drag pans the view.
        let response = ui.interact(
            ui.available_rect_before_wrap(),
            ui.id().with("scope-drag"),
            egui::Sense::drag(),
        );
        if response.dragged() {
            let side = ui
                .available_rect_before_wrap()
                .width()
                .min(ui.available_rect_before_wrap().height());
            let km_per_px = (2.0 * scope::MAX_RANGE_KM) / (side * self.zoom);
            self.pan_km += response.drag_delta() * km_per_px;
        }
        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            self.zoom = (self.zoom * (1.0 + scroll * 0.001)).clamp(0.05, 4.0);
        }

        // Collect GIS storms for scope drawing.
        let gis_storms: Vec<&crate::nhc::StormGis> = self
            .nhc_bundle
            .as_ref()
            .map(|b| b.gis_storms.iter().collect())
            .unwrap_or_default();

        // Collect storm metas for scope drawing.
        let storm_metas: Vec<&crate::nhc::StormMeta> = self
            .nhc_bundle
            .as_ref()
            .map(|b| b.metas.iter().collect())
            .unwrap_or_default();

        // Build overlay options from bundle and toggle state.
        let overlay_opts = scope::OverlayOptions {
            show_wind_probs_34kt: self.show_wind_probs_34kt,
            show_wind_probs_50kt: self.show_wind_probs_50kt,
            show_wind_probs_64kt: self.show_wind_probs_64kt,
            show_earliest_arrival: self.show_earliest_arrival,
            show_most_likely_arrival: self.show_most_likely_arrival,
            wind_probs_34kt: self
                .nhc_bundle
                .as_ref()
                .map(|b| b.wind_probs_34kt.clone())
                .unwrap_or_default(),
            wind_probs_50kt: self
                .nhc_bundle
                .as_ref()
                .map(|b| b.wind_probs_50kt.clone())
                .unwrap_or_default(),
            wind_probs_64kt: self
                .nhc_bundle
                .as_ref()
                .map(|b| b.wind_probs_64kt.clone())
                .unwrap_or_default(),
            earliest_arrival: self
                .nhc_bundle
                .as_ref()
                .map(|b| b.earliest_arrival.clone())
                .unwrap_or_default(),
            most_likely_arrival: self
                .nhc_bundle
                .as_ref()
                .map(|b| b.most_likely_arrival.clone())
                .unwrap_or_default(),
        };

        scope::draw_scope(
            ui,
            self.texture.as_ref(),
            self.scan.as_ref(),
            self.product,
            &self.borders,
            &self.alerts,
            &gis_storms,
            &storm_metas,
            self.current_site(),
            self.pan_km,
            self.zoom,
            &overlay_opts,
        );
    }
}
