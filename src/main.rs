use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};
use x11rb::rust_connection::RustConnection;
use image::{ImageBuffer, Rgba};

fn main() -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let width = screen.width_in_pixels;
    let height = screen.height_in_pixels;

    let img = conn.get_image(
        ImageFormat::Z_PIXMAP,
        screen.root,
        0,
        0,
        width,
        height,
        !0, // plane mask (all bits)
    )?
    .reply()?;

    let data = img.data;

    let mut buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width.into(), height.into());

    for (i, pixel) in buffer.pixels_mut().enumerate() {
        let offset = i * 4;
        if offset + 3 < data.len() {
            let b = data[offset];
            let g = data[offset + 1];
            let r = data[offset + 2];
            *pixel = Rgba([r, g, b, 255]);
        }
    }

    buffer.save("screenshot.png")?;
    println!("Скриншот сохранён как screenshot.png");



    Ok(())
}
