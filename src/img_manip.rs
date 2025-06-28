use {crate::geom::SrcRect, arboard::ImageData, std::borrow::Cow};

/// Returns a cropped version of `input`, defined by `rect`.
pub fn crop_image_data(input: &ImageData, rect: &SrcRect) -> ImageData<'static> {
    let mut pixels = vec![0; rect.w as usize * rect.h as usize * 4];
    copy_pixels(&input.bytes, &mut pixels, rect, input.width as u16);
    ImageData {
        width: rect.w as usize,
        height: rect.h as usize,
        bytes: Cow::Owned(pixels),
    }
}

/// Copies pixels row-by-row from source to destination, at the specified rectangle.
fn copy_pixels(src: &[u8], dst: &mut [u8], rect: &SrcRect, stride: u16) {
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
