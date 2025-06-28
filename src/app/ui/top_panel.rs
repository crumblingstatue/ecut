use {crate::app::EcutApp, eframe::egui};

pub fn ui(app: &mut EcutApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if app.img_recv.is_some() {
            ui.spinner();
        } else if ui
            .add(egui::Button::new("📋 Paste").shortcut_text("V"))
            .on_hover_text("Ctrl+V is broken thanks to egui :)")
            .clicked()
        {
            app.try_paste = true;
        }
        if let Some(img) = &mut app.img {
            let [x, c] =
                ui.input(|inp| [inp.key_pressed(egui::Key::X), inp.key_pressed(egui::Key::C)]);
            if let Some(rect) = &app.cut_rect
                && (ui
                    .add(egui::Button::new("✂ Cut").shortcut_text("X"))
                    .clicked()
                    || x)
            {
                img.cut(rect, ui.ctx());
                app.cut_rect = None;
            }
            if ui
                .add(egui::Button::new("🗐 Copy").shortcut_text("C"))
                .clicked()
                || c
            {
                arboard::Clipboard::new()
                    .unwrap()
                    .set_image(img.img.clone())
                    .unwrap();
            }
        }

        ui.checkbox(&mut app.fit, "Fit");
    });
    if let Some(err) = &app.err {
        ui.label(format!("Error: {err}"));
    }
}
