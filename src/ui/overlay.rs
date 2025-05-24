use crate::types::tag::UiTag;
use eframe::{
    egui::{self, Color32, Pos2, Stroke, ViewportBuilder, ViewportId},
    App,
};
use std::sync::{Arc, Mutex};

pub struct Overlay {
    tags: Arc<Mutex<Vec<UiTag>>>,
    pub(super) overlay_viewport_id: ViewportId,
    display_overlay: bool,
    fullscreen: bool,
}

impl Overlay {
    pub fn new(tags: &Arc<Mutex<Vec<UiTag>>>) -> Self {
        let tag_clone: Arc<Mutex<Vec<UiTag>>> = Arc::clone(tags);
        Self {
            tags: tag_clone,
            overlay_viewport_id: ViewportId::from_hash_of("Overlay"),
            display_overlay: false,
            fullscreen: false,
        }
    }

    pub(super) fn display_overlay(&self) -> bool {
        self.display_overlay
    }
    pub(super) fn set_display_overlay(&mut self, display: bool) {
        self.display_overlay = display;
    }
    pub(super) fn fullscreen(&self) -> bool {
        self.fullscreen
    }
    pub(super) fn set_fullscreen(&mut self, fs: bool) {
        self.fullscreen = fs;
    }
}

impl App for Overlay {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let tags_clone: Arc<Mutex<Vec<UiTag>>> = self.tags.clone();
        // let display = self.display_overlay;
        ctx.show_viewport_deferred(
            self.overlay_viewport_id,
            ViewportBuilder::default()
                .with_always_on_top()
                .with_mouse_passthrough(true)
                .with_resizable(false)
                .with_position(Pos2::new(0.0, 0.0))
                .with_taskbar(false)
                .with_transparent(true),
            move |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Deferred,
                    "This egui backend doesn't support multiple viewports"
                );

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        for t in tags_clone.lock().unwrap().iter() {
                            if !ui.input(|i: &egui::InputState| {
                                i.viewport().fullscreen.unwrap_or(false)
                            }) {
                                continue;
                            }
                            let rect = egui::Rect {
                                min: Pos2 {
                                    x: t.abs_bounding_box().x as f32,
                                    y: t.abs_bounding_box().y as f32,
                                },
                                max: Pos2 {
                                    x: (t.abs_bounding_box().x + t.abs_bounding_box().width) as f32,
                                    y: (t.abs_bounding_box().y + t.abs_bounding_box().height)
                                        as f32,
                                },
                            };
                            ui.painter().rect(
                                rect,
                                0,
                                Color32::TRANSPARENT,
                                Stroke::new(
                                    2.0,
                                    Color32::from_hex(if t.selected() {
                                        "#00FFFF"
                                    } else {
                                        "#00FF00"
                                    })
                                    .unwrap(),
                                ),
                                egui::StrokeKind::Middle,
                            );
                            ui.label(format!("{} / {}", t.tag_type().to_string(), t.selected()));
                        }
                    });
                ctx.request_repaint();
            },
        );
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }
}
