use std::{
    ffi::CString,
    sync::{atomic::AtomicU64, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::types::tag::{image_to_tags, into_mat, UiTag};
use eframe::egui::{self, Color32};
use egui::WidgetText;
use leptess::tesseract;
use xcap::{self, Window};

use super::overlay::Overlay;

pub struct MainMenu {
    window: Option<Arc<Mutex<Window>>>,
    tags: Arc<Mutex<Vec<UiTag>>>,
    capture_active: Arc<Mutex<bool>>,
    capture_interval: Arc<AtomicU64>,
    overlay: Overlay,
}

impl MainMenu {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let tag_arc: Arc<Mutex<Vec<UiTag>>> = Arc::new(Mutex::new(vec![]));
        Self {
            capture_active: Arc::new(Mutex::new(false)),
            window: None,
            tags: tag_arc.clone(),
            capture_interval: Arc::new(AtomicU64::new(500)),
            overlay: Overlay::new(&tag_arc),
        }
    }

    pub fn start_capture(&self) -> Result<(), Box<dyn std::error::Error>> {
        let running: Arc<Mutex<bool>> = Arc::clone(&self.capture_active);
        if self.window.is_none() {
            return Ok(());
        }
        let mut tess: tesseract::TessApi =
            tesseract::TessApi::new(Some("/usr/share/tessdata"), "eng")
                .expect("Failed to create TessApi");
        let key_cstr: CString =
            CString::new("tessedit_char_whitelist").expect("CString::new failed");
        let value_cstr: CString =
            CString::new("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-")
                .expect("CString::new failed");
        tess.raw
            .set_variable(&key_cstr, &value_cstr)
            .expect("Failed to set Tesseract char whitelist");

        let window_clone: Arc<Mutex<Window>> = Arc::clone(&self.window.as_ref().unwrap());
        let tag_clone: Arc<Mutex<Vec<UiTag>>> = Arc::clone(&self.tags);
        let interval: Arc<AtomicU64> = self.capture_interval.clone();
        thread::spawn(move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(
                    interval.load(std::sync::atomic::Ordering::Acquire),
                ));
                let window = window_clone.lock().unwrap();
                if window.is_minimized().unwrap() {
                    continue;
                }
                let image: xcap::image::ImageBuffer<xcap::image::Rgba<u8>, Vec<u8>> =
                    window.capture_image().unwrap();
                let tags: Vec<UiTag> = image_to_tags(&into_mat(&image), &mut tess)
                    .unwrap()
                    .iter()
                    .map(|t| {
                        UiTag::from_tag(
                            t,
                            window.x().unwrap_or_else(|_| 0)
                                - window.current_monitor().unwrap().x().unwrap_or_else(|_| 0),
                            window.y().unwrap_or_else(|_| 0)
                                - window.current_monitor().unwrap().y().unwrap_or_else(|_| 0),
                        )
                    })
                    .collect();
                *tag_clone.lock().unwrap() = tags;
            }
        });
        Ok(())
    }
}

impl eframe::App for MainMenu {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut selected: String = match &self.window {
            Some(window) => window
                .lock()
                .unwrap()
                .title()
                .unwrap_or_else(|_| "Unknown".to_string()),
            None => "no window selected".to_string(),
        };

        if self.overlay.display_overlay() {
            self.overlay.update(ctx, _frame);
        }

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let btn: egui::Response = ui.button("Start recognition");
            if btn.clicked() {
                let mut active: std::sync::MutexGuard<'_, bool> =
                    self.capture_active.lock().unwrap();
                *active = !*active;
                self.start_capture()
                    .expect("Failed to start screen capture");
            }
            if ui.button("Show/Hide Overlay").clicked() {
                ctx.send_viewport_cmd_to(
                    self.overlay.overlay_viewport_id,
                    egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop),
                );
                self.overlay
                    .set_display_overlay(!self.overlay.display_overlay());
            }
            if self.overlay.display_overlay() {
                if ui.button("Toggle fullScreen").clicked() {
                    self.overlay.set_fullscreen(!self.overlay.fullscreen());
                    ctx.send_viewport_cmd_to(
                        self.overlay.overlay_viewport_id,
                        egui::ViewportCommand::Fullscreen(self.overlay.fullscreen()),
                    );
                }
            }

            egui::ComboBox::from_id_salt("Select the Game")
                .selected_text(selected.clone())
                .show_ui(ui, |ui: &mut egui::Ui| {
                    let windows: Result<Vec<Window>, xcap::XCapError> = Window::all();
                    if windows.is_err() {
                        return;
                    }
                    for w in windows.unwrap().iter().filter(|f: &&Window| {
                        let name: Result<String, xcap::XCapError> = f.app_name();
                        name.is_ok_and(|f: String| !f.contains("wayland"))
                    }) {
                        let window_name: Result<String, xcap::XCapError> = w.title();
                        if let Ok(name) = window_name {
                            ui.selectable_value(
                                &mut selected,
                                name.clone(),
                                WidgetText::from(name.clone()),
                            );
                        }
                    }
                });
            let windows: Result<Vec<Window>, xcap::XCapError> = Window::all();
            if windows.is_ok() {
                for w in windows.unwrap() {
                    if w.title().unwrap() == selected {
                        self.window = Some(Arc::new(Mutex::new(w)));
                    }
                }
            }
        });
    }
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Color32::TRANSPARENT.to_normalized_gamma_f32()
    }
}
