use {
    crate::{
        app::{EcutApp, ImageBundle, UiState},
        geom::{SrcPos, SrcRect},
    },
    eframe::egui::{self, load::SizedTexture},
};

pub fn ui(app: &mut EcutApp, ui: &mut egui::Ui) {
    match &app.img {
        Some(img) => {
            image_ui(img, &mut app.ui_state, ui);
        }
        None => {
            ui.label("No image loaded");
        }
    }
}

fn image_ui(img: &ImageBundle, ui_state: &mut UiState, ui: &mut egui::Ui) {
    egui::ScrollArea::both().show(ui, |ui| {
        let size = if ui_state.fit {
            ui.available_size()
        } else {
            img.tex.size_vec2()
        };
        let (orig_w, orig_h) = (img.img.width, img.img.height);
        let (h_ratio, v_ratio) = (size.x / orig_w as f32, size.y / orig_h as f32);
        let re = ui.add(
            egui::Image::new(SizedTexture::new(img.tex.id(), size))
                .sense(egui::Sense::click_and_drag()),
        );
        if let Some(rect) = &ui_state.cut_rect {
            ui.painter_at(re.rect).rect(
                egui::Rect {
                    min: egui::pos2(
                        f32::from(rect.x).mul_add(h_ratio, re.rect.min.x),
                        f32::from(rect.y).mul_add(v_ratio, re.rect.min.y),
                    ),
                    max: egui::pos2(
                        (f32::from(rect.x) + f32::from(rect.w)).mul_add(h_ratio, re.rect.min.x),
                        (f32::from(rect.y) + f32::from(rect.h)).mul_add(v_ratio, re.rect.min.y),
                    ),
                },
                egui::CornerRadius::ZERO,
                egui::Color32::from_rgba_unmultiplied(255, 0, 0, 25),
                egui::Stroke::new(1.0, egui::Color32::RED),
                egui::StrokeKind::Inside,
            );
        }
        let (ptr_pos, any_down, any_released) = ui.input(|inp| {
            (
                inp.pointer.latest_pos(),
                inp.pointer.any_down(),
                inp.pointer.any_released(),
            )
        });
        if let Some(mut pos) = ptr_pos {
            pos -= re.rect.min.to_vec2();
            pos = egui::pos2(pos.x / h_ratio, pos.y / v_ratio);
            ui_state.img_cursor_pos = Some(pos);
            if re.hovered() && any_down && ui_state.click_origin.is_none() {
                ui_state.click_origin = Some(SrcPos {
                    x: pos.x as u16,
                    y: pos.y as u16,
                });
            }
            if any_released {
                ui_state.click_origin = None;
            }
            if let Some(orig) = &ui_state.click_origin
                && let Some(new_w) = (pos.x as u16).checked_sub(orig.x)
                && let Some(new_h) = (pos.y as u16).checked_sub(orig.y)
            {
                ui_state.cut_rect = Some(SrcRect {
                    x: orig.x,
                    y: orig.y,
                    w: new_w,
                    h: new_h,
                });
            }
        }
    });
}
