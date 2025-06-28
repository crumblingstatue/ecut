use {
    crate::geom::{SrcPos, SrcRect},
    arboard::{Clipboard, ImageData},
    eframe::egui::{self, ColorImage, TextureHandle, TextureOptions},
    std::sync::mpsc::TryRecvError,
};

mod ui {
    pub mod bottom_panel;
    pub mod central_panel;
    pub mod top_panel;
}

#[derive(Default)]
pub struct EcutApp {
    img: Option<ImageBundle>,
    img_recv: Option<ImgRecv>,
    ui_state: UiState,
}

struct UiState {
    fit: bool,
    img_cursor_pos: Option<egui::Pos2>,
    cut_rect: Option<SrcRect>,
    click_origin: Option<SrcPos>,
    err: Option<String>,
    /// Try to paste image data
    try_paste: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            fit: true,
            img_cursor_pos: Default::default(),
            cut_rect: Default::default(),
            click_origin: Default::default(),
            err: Default::default(),
            try_paste: Default::default(),
        }
    }
}

struct ImageBundle {
    /// Arboard image data
    img: ImageData<'static>,
    /// Egui texture handle
    tex: TextureHandle,
}

impl ImageBundle {
    fn cut(&mut self, rect: &SrcRect, ctx: &egui::Context) {
        self.img = crate::img_manip::crop_image_data(&self.img, rect);
        self.tex = alloc_tex_from_img(&self.img, ctx);
    }
}

type ImgRecv = std::sync::mpsc::Receiver<Result<ImageBundle, arboard::Error>>;

fn try_load_img_from_clipboard_async(ctx: &egui::Context) -> Result<ImgRecv, arboard::Error> {
    let (send, recv) = std::sync::mpsc::channel();
    let mut cb = Clipboard::new()?;
    let ctx = ctx.clone();
    std::thread::spawn(move || {
        send.send(try_load_img_from_clipboard(&mut cb, &ctx))
            .unwrap();
    });
    Ok(recv)
}

fn alloc_tex_from_img(img: &ImageData, ctx: &egui::Context) -> TextureHandle {
    let size = [img.width, img.height];
    let color_img = ColorImage::from_rgba_unmultiplied(size, &img.bytes);
    ctx.load_texture("", color_img, TextureOptions::default())
}

fn try_load_img_from_clipboard(
    cb: &mut Clipboard,
    ctx: &egui::Context,
) -> Result<ImageBundle, arboard::Error> {
    match cb.get_image() {
        Ok(img) => Ok(ImageBundle {
            tex: alloc_tex_from_img(&img, ctx),
            img,
        }),
        Err(e) => Err(e),
    }
}

impl eframe::App for EcutApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.ui_state.try_paste {
            self.ui_state.try_paste = false;
            self.ui_state.err = None;
            match try_load_img_from_clipboard_async(ctx) {
                Ok(recv) => {
                    self.img_recv = Some(recv);
                }
                Err(e) => {
                    self.ui_state.err = Some(e.to_string());
                }
            }
        } else {
            // FIXME: egui eats paste events. No way to know if an image was pasted.
            // https://github.com/emilk/egui/issues/4065
            // As a workaround, we just use V
            if ctx.input(|inp| inp.key_pressed(egui::Key::V)) {
                self.ui_state.try_paste = true;
            }
        }
        if let Some(recv) = &self.img_recv {
            match recv.try_recv() {
                Ok(result) => match result {
                    Ok(tex) => {
                        self.img = Some(tex);
                    }
                    Err(e) => {
                        self.ui_state.err = Some(e.to_string());
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.img_recv = None;
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui::top_panel::ui(self, ui);
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui::bottom_panel::ui(self, ui);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::central_panel::ui(self, ui);
        });
    }
}
