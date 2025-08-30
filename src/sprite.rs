use {
    crate::App,
    image::ImageReader,
    std::{io::Cursor, num::NonZeroU32},
};

pub fn map() -> App<(image::RgbaImage, NonZeroU32, NonZeroU32)> {
    let sprites = ImageReader::new(Cursor::new(include_bytes!("../sprites.png")))
        .with_guessed_format()?
        .decode()?;

    let image = sprites.to_rgba8();
    let width = NonZeroU32::new(sprites.width()).ok_or("zero width")?;
    let height = NonZeroU32::new(sprites.height()).ok_or("zero height")?;
    Ok((image, width, height))
}
