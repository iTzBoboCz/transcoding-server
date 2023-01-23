use axum::http::StatusCode;
use infer::Type;
use magick_rust::{bindings, magick_wand_genesis, MagickWand, PixelWand, ResourceType};
use magick_rust::{MagickError, ToMagick};
use std::path::Path;
use std::sync::Once;

#[derive(Default)]
pub enum SupportedImageFormats {
    #[default]
    WebP,
    JPEG,
    PNG,
    GIF,
}

// Used to make sure MagickWand is initialized exactly once. Note that we
// do not bother shutting down, we simply exit when the tests are done.
static START: Once = Once::new();

pub fn init() -> MagickWand {
    START.call_once(|| {
        magick_wand_genesis();
    });

    MagickWand::new()
}

pub async fn lower_bitrate(
    path: &Path,
    mime_type: &str,
) -> Result<tempfile::NamedTempFile, StatusCode> {
    let wand = init();

    wand.read_image(
        path.to_str()
            .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    let height = wand.get_image_height();
    let width = wand.get_image_width();

    wand.thumbnail_image(
        (width / 2) + (width % 2 != 0) as usize,
        (height / 2) + (height % 2 != 0) as usize,
    );

    let file = tempfile::Builder::new()
        .suffix(&format!(".{}", mime_type))
        .tempfile();

    let output = match file {
        Ok(f) => Ok(f),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }?;

    if wand.write_image(output.path().to_str().unwrap()).is_err() {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    };

    Ok(output)
}

pub fn get_dimensions(path: &Path) -> (usize, usize) {
    let wand = init();

    (wand.get_image_width(), wand.get_image_height())
}
