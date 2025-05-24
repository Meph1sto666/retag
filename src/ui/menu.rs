use std::{sync::{Arc, Mutex}, thread, time::Duration};

use crate::types::tag::Tag;
use eframe::egui;
use egui::WidgetText;
use xcap::{self, Window};

pub struct MainMenu {
    window: Option<Arc<Mutex<Window>>>,
    tags: Arc<Mutex<Vec<Tag>>>,
    capture_active: Arc<Mutex<bool>>,
}

impl MainMenu {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
			capture_active: Arc::new(Mutex::new(false)),
            window: None,
            tags: Arc::new(Mutex::new(vec![])),
        }
    }

	pub fn start_capture(&self) -> Result<(), Box<dyn std::error::Error>> {
		if self.window.is_none() {
			return Ok(());
		}
		let window_clone: Arc<Mutex<Window>> = Arc::clone(&self.window.as_ref().unwrap());
		let tag_clone: Arc<Mutex<Vec<Tag>>> = Arc::clone(&self.tags);
		let running: Arc<Mutex<bool>> = Arc::clone(&self.capture_active);
		thread::spawn(move || {
			while *running.lock().unwrap() {
				thread::sleep(Duration::from_millis(750));
				println!("screenshot taken");
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

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
			let btn: egui::Response = ui.button("Start recognition");
			if btn.clicked() {
				let mut locked: std::sync::MutexGuard<'_, bool> = self.capture_active.lock().unwrap();
				*locked = !*locked;
				self.start_capture();
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
}
