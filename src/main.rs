use {crate::app::EcutApp, eframe::NativeOptions};

mod app;

fn main() {
    eframe::run_native(
        "ecut",
        NativeOptions::default(),
        Box::new(move |_cc| Ok(Box::new(EcutApp::new()))),
    )
    .unwrap();
}
