use {crate::app::EcutApp, eframe::egui};

pub fn ui(app: &mut EcutApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if let Some(pos) = &app.img_cursor_pos {
            ui.label(format!("Cursor pos: {pos:?}"));
        }
        if let Some(rect) = &mut app.cut_rect
            && let Some(img) = &app.img
        {
            let [tex_w, tex_h] = img.tex.size().map(|v| v as u16);
            ui.label("x");
            ui.add(egui::Slider::new(&mut rect.x, 0..=tex_w).drag_value_speed(1.0));
            ui.label("y");
            ui.add(egui::Slider::new(&mut rect.y, 0..=tex_h).drag_value_speed(1.0));
            ui.label("w");
            ui.add(egui::Slider::new(&mut rect.w, 0..=tex_w).drag_value_speed(1.0));
            ui.label("h");
            ui.add(egui::Slider::new(&mut rect.h, 0..=tex_h).drag_value_speed(1.0));
        }
    });
}
