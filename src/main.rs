use {
    arboard::Clipboard,
    eframe::{
        NativeOptions,
        egui::{self, ColorImage, TextureHandle, TextureOptions, load::SizedTexture},
    },
};

struct EcutApp {
    clipboard: Clipboard,
    tex: Option<TextureHandle>,
    err: Option<String>,
    /// Try to paste image data
    try_paste: bool,
}

fn try_load_img_from_clipboard(
    cb: &mut Clipboard,
    ctx: &egui::Context,
) -> Result<TextureHandle, arboard::Error> {
    match cb.get_image() {
        Ok(img) => {
            let size = [img.width, img.height];
            let color_img = ColorImage::from_rgba_unmultiplied(size, &img.bytes);
            let tex = ctx.load_texture("", color_img, TextureOptions::default());
            Ok(tex)
        }
        Err(e) => Err(e),
    }
}

fn main() {
    eframe::run_native(
        "ecut",
        NativeOptions::default(),
        Box::new(move |_cc| {
            Ok(Box::new(EcutApp {
                clipboard: arboard::Clipboard::new().unwrap(),
                tex: None,
                err: None,
                try_paste: true,
            }))
        }),
    )
    .unwrap();
}

impl eframe::App for EcutApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.try_paste {
            self.try_paste = false;
            match try_load_img_from_clipboard(&mut self.clipboard, ctx) {
                Ok(tex) => {
                    self.tex = Some(tex);
                }
                Err(e) => {
                    self.err = Some(e.to_string());
                }
            }
        } else {
            // FIXME: egui eats paste events. No way to know if an image was pasted.
            // https://github.com/emilk/egui/issues/4065
            // As a workaround, we use F5 for "refresh"
            if ctx.input(|inp| inp.key_pressed(egui::Key::F5)) {
                self.try_paste = true;
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.label("Press F5 to refresh, ctrl+V is broken thanks to egui :)");
        });
        egui::CentralPanel::default().show(ctx, |ui| match &self.tex {
            Some(tex) => {
                ui.add(egui::Image::new(SizedTexture::new(
                    tex.id(),
                    tex.size_vec2(),
                )));
            }
            None => {
                ui.label("No image loaded");
                if let Some(err) = &self.err {
                    ui.label(err);
                }
            }
        });
    }
}
