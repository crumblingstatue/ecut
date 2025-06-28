use {
    crate::app::EcutApp,
    eframe::{NativeOptions, egui},
};

mod app;
mod geom;
mod img_manip;

fn main() {
    let native_opts = NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder.with_inner_size(egui::vec2(848., 600.))
        })),
        ..Default::default()
    };
    eframe::run_native(
        "ecut",
        native_opts,
        Box::new(move |_cc| Ok(Box::new(EcutApp::default()))),
    )
    .unwrap();
}
