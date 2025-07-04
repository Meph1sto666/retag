mod types;
mod ui;
mod core;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eframe::run_native(
        "Re:Tag",
        eframe::NativeOptions::default(),
        Box::new(|cc: &eframe::CreationContext<'_>| Ok(Box::new(ui::menu::MainMenu::new(cc)))),
    )?;
    Ok(())
}
