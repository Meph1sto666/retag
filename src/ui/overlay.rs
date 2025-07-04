use crate::{core::calculator::Calculator, types::tag::UiTag};
use eframe::{
    App,
    egui::{self, Align2, Color32, FontFamily, Pos2, Stroke, ViewportBuilder, ViewportId},
};
use getset::{Getters, Setters};
use std::sync::{Arc, Mutex};

#[derive(Getters, Setters)]
#[get = "pub"]
#[set = "pub"]
pub struct Overlay {
    tags: Arc<Mutex<Vec<UiTag>>>,
    pub(super) overlay_viewport_id: ViewportId,
    display_overlay: bool,
    fullscreen: bool,
    calculator: Arc<Mutex<Calculator>>,
}

impl Overlay {
    pub fn new(tags: &Arc<Mutex<Vec<UiTag>>>, calculator: &Arc<Mutex<Calculator>>) -> Self {
        let tag_clone: Arc<Mutex<Vec<UiTag>>> = tags.clone();
        let calc_clone: Arc<Mutex<Calculator>> = calculator.clone();
        Self {
            tags: tag_clone,
            calculator: calc_clone,
            overlay_viewport_id: ViewportId::from_hash_of("Overlay"),
            display_overlay: false,
            fullscreen: false,
        }
    }
}

impl App for Overlay {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let tags_clone: Arc<Mutex<Vec<UiTag>>> = self.tags.clone();
        let calc_clone: Arc<Mutex<Calculator>> = self.calculator.clone();
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
                                Stroke::new(2.0, Color32::from_hex("#00FF00").unwrap()),
                                egui::StrokeKind::Outside,
                            );
                            ui.painter().text(
                                rect.min,
                                Align2::LEFT_BOTTOM,
                                format!("{}", t.tag_type().to_string()),
                                egui::FontId {
                                    size: 16.0,
                                    family: FontFamily::Monospace,
                                },
                                Color32::from_hex(if t.selected() { "#00ff00" } else { "#FF0000" })
                                    .unwrap(),
                            );
                        }

                        for res in calc_clone.lock().unwrap().evaluate(tags_clone.clone()) {
                            ui.label(format!("{:?}", res));
                        }
                    });
                ctx.request_repaint();
            },
        );
    }
}

fn draw_ops() {
    
}