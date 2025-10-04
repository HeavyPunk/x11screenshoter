use anyhow::Result;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};
use x11rb::rust_connection::RustConnection;
use image::{ImageBuffer, Rgba};
use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use std::io::Write;

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

    // Prepare SPI using spidev
    let mut spi = Spidev::open("/dev/spidev3.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(8_000_000)
        .mode(SpiModeFlags::SPI_MODE_0)
        .build();
    spi.configure(&options)?;

    // Convert buffer to raw RGB data for SPI (strip alpha)
    let mut spi_data = Vec::with_capacity((width * height * 3) as usize);
    for pixel in buffer.pixels() {
        spi_data.push(pixel[0]); // R
        spi_data.push(pixel[1]); // G
        spi_data.push(pixel[2]); // B
    }

    // Send data over SPI in chunks
    let chunk_size = 4096;
    for chunk in spi_data.chunks(chunk_size) {
        spi.write_all(chunk)?;
    }

    println!("Изображение отправлено на дисплей через SPI");

    Ok(())
}

