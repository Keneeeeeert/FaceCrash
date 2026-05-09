mod app;
mod bg_remover;
mod camera;

use std::env;
use std::path::PathBuf;

use app::App;
use camera::CameraSource;

const FONT_PATHS: &[&str] = &[
    "C:\\Windows\\Fonts\\msyh.ttc",
    "C:\\Windows\\Fonts\\msyhbd.ttc",
    "C:\\Windows\\Fonts\\simhei.ttf",
    "C:\\Windows\\Fonts\\simsun.ttc",
    "C:\\Windows\\Fonts\\msgothic.ttc",
];

fn try_load_chinese_font() -> Option<Vec<u8>> {
    for path in FONT_PATHS {
        if let Ok(data) = std::fs::read(path) {
            println!("Loaded font: {}", path);
            return Some(data);
        }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // Go up from target/release/ to project root, then to C:\LZXify\images
    let images_dir = exe_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("images"))
        .unwrap_or_else(|| exe_dir.join("images"));
    std::fs::create_dir_all(&images_dir).ok();

    println!("Opening camera...");
    let camera = CameraSource::new(640, 480)?;

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([680.0, 620.0])
            .with_resizable(false)
            .with_title("透明背景 PNG 采集器"),
        ..Default::default()
    };

    println!("Starting UI...");
    eframe::run_native(
        "png-collector",
        native_options,
        Box::new(|cc| {
            // Load Chinese font
            if let Some(font_data) = try_load_chinese_font() {
                let mut fonts = eframe::egui::FontDefinitions::default();
                fonts
                    .font_data
                    .insert("chinese".to_owned(), std::sync::Arc::new(eframe::egui::FontData::from_owned(font_data)));
                fonts
                    .families
                    .entry(eframe::egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".to_owned());
                fonts
                    .families
                    .entry(eframe::egui::FontFamily::Monospace)
                    .or_default()
                    .push("chinese".to_owned());
                cc.egui_ctx.set_fonts(fonts);
            }
            Ok(Box::new(App::new(camera, images_dir)))
        }),
    )?;

    Ok(())
}
