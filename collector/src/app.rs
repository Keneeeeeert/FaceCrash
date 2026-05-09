use std::path::PathBuf;
use std::time::Instant;
use std::thread;

use eframe::egui::{self, Color32};
use image::RgbaImage;

use crate::bg_remover::{remove_background_python, next_filename};
use crate::camera::{CameraSource, Frame};

enum CaptureState {
    Idle,
    Countdown { start: Instant, last_num: i32 },
    Processing,
    Done(String),
    Error(String),
}

pub struct App {
    camera: CameraSource,
    latest_frame: Option<Frame>,
    camera_tex: Option<egui::TextureHandle>,
    result_tex: Option<egui::TextureHandle>,
    state: CaptureState,
    last_result: Option<(Vec<u8>, u32, u32)>,
    result_receiver: Option<std::sync::mpsc::Receiver<Result<(Vec<u8>, String), String>>>,
    images_dir: PathBuf,
}

impl App {
    pub fn new(
        camera: CameraSource,
        images_dir: PathBuf,
    ) -> Self {
        Self {
            camera,
            latest_frame: None,
            camera_tex: None,
            result_tex: None,
            state: CaptureState::Idle,
            last_result: None,
            result_receiver: None,
            images_dir,
        }
    }

    fn start_capture(&mut self) {
        if matches!(self.state, CaptureState::Idle) {
            self.state = CaptureState::Countdown {
                start: Instant::now(),
                last_num: 3,
            };
        }
    }

    fn start_processing(&mut self) {
        if let Some(ref frame) = self.latest_frame {
            let rgb = frame.data.clone();
            let w = frame.width;
            let h = frame.height;
            let fname = next_filename(&self.images_dir);
            let images_dir = self.images_dir.clone();

            let (tx, rx) = std::sync::mpsc::channel();

            thread::Builder::new()
                .name("bg-removal".into())
                .spawn(move || {
                    let result = remove_background_python(&rgb, w, h);
                    match result {
                        Ok(rgba) => {
                            let path = images_dir.join(&fname);
                            if let Some(img) = RgbaImage::from_raw(w, h, rgba.clone()) {
                                let _ = img.save(&path);
                            }
                            let _ = tx.send(Ok((rgba, fname)));
                        }
                        Err(e) => {
                            let _ = tx.send(Err(format!("{}", e)));
                        }
                    }
                })
                .ok();

            self.result_receiver = Some(rx);
            self.state = CaptureState::Processing;
        }
    }

    fn check_processing_result(&mut self) -> bool {
        if let Some(ref rx) = self.result_receiver {
            if let Ok(result) = rx.try_recv() {
                self.result_receiver = None;
                match result {
                    Ok((rgba, fname)) => {
                        self.last_result = Some((rgba.clone(), self.latest_frame.as_ref().map_or(0, |f| f.width), self.latest_frame.as_ref().map_or(0, |f| f.height)));
                        self.state = CaptureState::Done(fname);
                    }
                    Err(e) => {
                        self.state = CaptureState::Error(e);
                    }
                }
                return true;
            }
        }
        false
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll new camera frame
        if let Some(frame) = self.camera.poll_frame() {
            self.latest_frame = Some(frame);
        }

        // Check processing result
        self.check_processing_result();

        // Handle countdown
        if let CaptureState::Countdown { start, ref mut last_num } = self.state {
            let elapsed = start.elapsed().as_secs_f32();
            let current_num = (3.0 - elapsed).ceil() as i32;
            if current_num < *last_num && current_num > 0 {
                *last_num = current_num;
            }
            if elapsed >= 3.0 {
                self.start_processing();
            }
        }

        // Request repaint continuously
        ctx.request_repaint();

        // Dark theme
        let dark_bg = Color32::from_rgb(26, 26, 46);
        let accent = Color32::from_rgb(233, 69, 96);
        let text_secondary = Color32::from_rgb(136, 136, 136);

        // Set dark background
        let mut style: egui::Style = (*ctx.style()).clone();
        style.visuals.panel_fill = dark_bg;
        style.visuals.window_fill = dark_bg;
        style.visuals.extreme_bg_color = dark_bg;
        ctx.set_style(style);

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                // Title
                ui.add_space(16.0);
                ui.colored_label(
                    egui::Color32::from_rgb(233, 69, 96),
                    egui::RichText::new("透明背景 PNG 采集器")
                        .font(egui::FontId::new(18.0, egui::FontFamily::Proportional))
                        .strong(),
                );
                ui.add_space(10.0);

                // Camera viewport area
                let vp_size = egui::vec2(640.0, 480.0);
                let (rect, _) = ui.allocate_exact_size(vp_size, egui::Sense::hover());

                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

                // Draw camera frame or placeholder
                if let Some(ref frame) = self.latest_frame {
                    let color_img = egui::ColorImage::from_rgb(
                        [frame.width as usize, frame.height as usize],
                        &frame.data,
                    );

                    let tex_options = egui::TextureOptions {
                        magnification: egui::TextureFilter::Linear,
                        minification: egui::TextureFilter::Linear,
                        ..Default::default()
                    };

                    if let Some(ref mut tex) = self.camera_tex {
                        tex.set(color_img, tex_options);
                    } else {
                        let tex = ctx.load_texture("camera", color_img, tex_options);
                        self.camera_tex = Some(tex);
                    }

                    if let Some(ref tex) = self.camera_tex {
                        ui.painter().image(
                            tex.id(),
                            rect,
                            uv,
                            egui::Color32::WHITE,
                        );
                    }
                } else {
                    // Black placeholder
                    ui.painter().rect_filled(rect, 0.0, Color32::BLACK);
                }

                // Overlay for countdown
                if let CaptureState::Countdown { last_num, .. } = self.state {
                    if last_num > 0 {
                        // Semi-transparent overlay
                        ui.painter().rect_filled(
                            rect,
                            0.0,
                            Color32::from_black_alpha(217),
                        );
                        // Countdown number
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            last_num.to_string(),
                            egui::FontId::new(80.0, egui::FontFamily::Proportional),
                            Color32::WHITE,
                        );
                    }
                }

                // Overlay for processing
                if matches!(self.state, CaptureState::Processing) {
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_black_alpha(217),
                    );
                    ui.painter().text(
                        egui::pos2(rect.center().x, rect.center().y - 20.0),
                        egui::Align2::CENTER_CENTER,
                        "AI 抠图中...",
                        egui::FontId::new(20.0, egui::FontFamily::Proportional),
                        accent,
                    );
                    // Spinner
                    let spinner_size = 40.0;
                    let spinner_rect = egui::Rect::from_center_size(
                        egui::pos2(rect.center().x, rect.center().y + 30.0),
                        egui::vec2(spinner_size, spinner_size),
                    );
                    let spinner = egui::widgets::Spinner::new();
                    ui.put(spinner_rect, spinner);
                    ui.painter().text(
                        egui::pos2(rect.center().x, rect.center().y + 60.0),
                        egui::Align2::CENTER_CENTER,
                        "处理中，请稍候...",
                        egui::FontId::new(10.0, egui::FontFamily::Proportional),
                        text_secondary,
                    );
                }

                ui.add_space(16.0);

                // Capture button
                let btn_text = match self.state {
                    CaptureState::Idle => "拍摄",
                    CaptureState::Done(_) | CaptureState::Error(_) => "继续拍摄",
                    _ => "拍摄中...",
                };

                let enabled = matches!(
                    self.state,
                    CaptureState::Idle | CaptureState::Done(_) | CaptureState::Error(_)
                );

                let btn_response = if enabled {
                    ui.add_enabled(true, egui::Button::new(
                        egui::RichText::new(btn_text)
                            .font(egui::FontId::new(14.0, egui::FontFamily::Proportional))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .fill(accent)
                    .min_size(egui::vec2(120.0, 42.0)))
                } else {
                    ui.add_enabled(false, egui::Button::new(
                        egui::RichText::new(btn_text)
                            .font(egui::FontId::new(14.0, egui::FontFamily::Proportional))
                            .strong()
                            .color(Color32::WHITE),
                    )
                    .min_size(egui::vec2(120.0, 42.0)))
                };

                if btn_response.clicked() {
                    self.state = CaptureState::Idle;
                    self.start_capture();
                }

                ui.add_space(12.0);

                // Status text
                let status = match &self.state {
                    CaptureState::Idle => String::new(),
                    CaptureState::Countdown { .. } => String::new(),
                    CaptureState::Processing => String::new(),
                    CaptureState::Done(fname) => format!("已保存: {}", fname),
                    CaptureState::Error(msg) => format!("失败: {}", msg),
                };

                if !status.is_empty() {
                    ui.label(
                        egui::RichText::new(status)
                            .font(egui::FontId::new(9.0, egui::FontFamily::Proportional))
                            .color(text_secondary),
                    );
                }

                // Show last result thumbnail
                if let Some(ref result) = self.last_result {
                    ui.add_space(10.0);
                    let thumb_w = 160.0;
                    let thumb_h = thumb_w * result.2 as f32 / result.1 as f32;
                    let thumb_size = egui::vec2(thumb_w, thumb_h);

                    let color_img = egui::ColorImage::from_rgba_unmultiplied(
                        [result.1 as usize, result.2 as usize],
                        &result.0,
                    );

                    let tex_options = egui::TextureOptions {
                        magnification: egui::TextureFilter::Linear,
                        minification: egui::TextureFilter::Linear,
                        ..Default::default()
                    };

                    if let Some(ref mut tex) = self.result_tex {
                        tex.set(color_img, tex_options);
                    } else {
                        let tex = ctx.load_texture("result", color_img, tex_options);
                        self.result_tex = Some(tex);
                    }

                    if let Some(ref tex) = self.result_tex {
                        let (rect, _) = ui.allocate_exact_size(thumb_size, egui::Sense::hover());
                        ui.painter().image(
                            tex.id(),
                            rect,
                            uv,
                            egui::Color32::WHITE,
                        );
                    }
                }
            });
        });
    }
}
