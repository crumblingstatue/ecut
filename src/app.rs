use {
    arboard::{Clipboard, ImageData},
    eframe::egui::{self, ColorImage, TextureHandle, TextureOptions, load::SizedTexture},
    std::{borrow::Cow, sync::mpsc::TryRecvError},
};

pub struct EcutApp {
    img: Option<ImageBundle>,
    err: Option<String>,
    /// Try to paste image data
    try_paste: bool,
    img_recv: Option<ImgRecv>,
    fit: bool,
    img_cursor_pos: Option<egui::Pos2>,
    cut_rect: Option<SourceRect>,
    click_origin: Option<SourcePos>,
}

struct SourcePos {
    x: u16,
    y: u16,
}

struct ImageBundle {
    /// Arboard image data
    img: ImageData<'static>,
    /// Egui texture handle
    tex: TextureHandle,
}

impl ImageBundle {
    fn cut(&mut self, rect: &SourceRect, ctx: &egui::Context) {
        self.img = resize_image_data(&self.img, rect);
        self.tex = alloc_tex_from_img(&self.img, ctx);
    }
}

fn resize_image_data(input: &ImageData, rect: &SourceRect) -> ImageData<'static> {
    let mut pixels = vec![0; rect.w as usize * rect.h as usize * 4];
    copy_pixels(&input.bytes, &mut pixels, rect, input.width as u16);
    ImageData {
        width: rect.w as usize,
        height: rect.h as usize,
        bytes: Cow::Owned(pixels),
    }
}

/// Copy pixels row-by-row from source to destination, at the specified rectangle
fn copy_pixels(src: &[u8], dst: &mut [u8], rect: &SourceRect, stride: u16) {
    let mut dx = 0;
    let mut dy = 0;
    for rgba in dst.as_chunks_mut().0 {
        *rgba = index_img(src, rect.x + dx, rect.y + dy, stride);
        dx += 1;
        if dx == rect.w {
            dx = 0;
            dy += 1;
        }
    }
}

fn index_img(src: &[u8], x: u16, y: u16, stride: u16) -> [u8; 4] {
    let flat_pos: usize = y as usize * stride as usize + x as usize;
    src.as_chunks().0[flat_pos]
}

struct SourceRect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
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

impl EcutApp {
    pub fn new() -> Self {
        Self {
            img: None,
            err: None,
            try_paste: true,
            img_recv: None,
            fit: true,
            img_cursor_pos: None,
            cut_rect: None,
            click_origin: None,
        }
    }
}

impl eframe::App for EcutApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if self.try_paste {
            self.try_paste = false;
            self.err = None;
            match try_load_img_from_clipboard_async(ctx) {
                Ok(recv) => {
                    self.img_recv = Some(recv);
                }
                Err(e) => {
                    self.err = Some(e.to_string());
                }
            }
        } else {
            // FIXME: egui eats paste events. No way to know if an image was pasted.
            // https://github.com/emilk/egui/issues/4065
            // As a workaround, we just use V
            if ctx.input(|inp| inp.key_pressed(egui::Key::V)) {
                self.try_paste = true;
            }
        }
        if let Some(recv) = &self.img_recv {
            match recv.try_recv() {
                Ok(result) => match result {
                    Ok(tex) => {
                        self.img = Some(tex);
                    }
                    Err(e) => {
                        self.err = Some(e.to_string());
                    }
                },
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.img_recv = None;
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.img_recv.is_some() {
                    ui.spinner();
                } else if ui
                    .add(egui::Button::new("📋 Paste").shortcut_text("V"))
                    .on_hover_text("Ctrl+V is broken thanks to egui :)")
                    .clicked()
                {
                    self.try_paste = true;
                }
                if let Some(img) = &mut self.img {
                    let [x, c] = ui.input(|inp| {
                        [inp.key_pressed(egui::Key::X), inp.key_pressed(egui::Key::C)]
                    });
                    if let Some(rect) = &self.cut_rect
                        && (ui
                            .add(egui::Button::new("✂ Cut").shortcut_text("X"))
                            .clicked()
                            || x)
                    {
                        img.cut(rect, ctx);
                        self.cut_rect = None;
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

                ui.checkbox(&mut self.fit, "Fit");
            });
            if let Some(err) = &self.err {
                ui.label(format!("Error: {err}"));
            }
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(pos) = &self.img_cursor_pos {
                    ui.label(format!("Cursor pos: {pos:?}"));
                }
                if let Some(rect) = &mut self.cut_rect
                    && let Some(img) = &self.img
                {
                    let [tex_w, tex_h] = img.tex.size().map(|v| v as u16);
                    ui.label("x");
                    ui.add(egui::Slider::new(&mut rect.x, 0..=tex_w));
                    ui.label("y");
                    ui.add(egui::Slider::new(&mut rect.y, 0..=tex_h));
                    ui.label("w");
                    ui.add(egui::Slider::new(&mut rect.w, 0..=tex_w));
                    ui.label("h");
                    ui.add(egui::Slider::new(&mut rect.h, 0..=tex_h));
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| match &self.img {
            Some(img) => {
                egui::ScrollArea::both().show(ui, |ui| {
                    let size = if self.fit {
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
                    ui.painter().debug_text(
                        re.rect.min,
                        egui::Align2::LEFT_TOP,
                        egui::Color32::GREEN,
                        format!("{h_ratio}, {v_ratio}"),
                    );
                    if let Some(rect) = &self.cut_rect {
                        ui.painter_at(re.rect).rect(
                            egui::Rect {
                                min: egui::pos2(
                                    re.rect.min.x + rect.x as f32 * h_ratio,
                                    re.rect.min.y + rect.y as f32 * v_ratio,
                                ),
                                max: egui::pos2(
                                    re.rect.min.x + (rect.x as f32 + rect.w as f32) * h_ratio,
                                    re.rect.min.y + (rect.y as f32 + rect.h as f32) * v_ratio,
                                ),
                            },
                            egui::CornerRadius::ZERO,
                            egui::Color32::from_rgba_unmultiplied(255, 0, 0, 25),
                            egui::Stroke::new(1.0, egui::Color32::RED),
                            egui::StrokeKind::Inside,
                        );
                    }
                    let (ptr_pos, any_down, any_released) = ctx.input(|inp| {
                        (
                            inp.pointer.latest_pos(),
                            inp.pointer.any_down(),
                            inp.pointer.any_released(),
                        )
                    });
                    if let Some(mut pos) = ptr_pos {
                        pos -= re.rect.min.to_vec2();
                        pos = egui::pos2(pos.x / h_ratio, pos.y / v_ratio);
                        self.img_cursor_pos = Some(pos);
                        if re.hovered() && any_down && self.click_origin.is_none() {
                            self.click_origin = Some(SourcePos {
                                x: pos.x as u16,
                                y: pos.y as u16,
                            });
                        }
                        if any_released {
                            self.click_origin = None;
                        }
                        if let Some(orig) = &self.click_origin
                            && let Some(new_w) = (pos.x as u16).checked_sub(orig.x)
                            && let Some(new_h) = (pos.y as u16).checked_sub(orig.y)
                        {
                            self.cut_rect = Some(SourceRect {
                                x: orig.x,
                                y: orig.y,
                                w: new_w,
                                h: new_h,
                            });
                        }
                    }
                });
            }
            None => {
                ui.label("No image loaded");
            }
        });
    }
}
