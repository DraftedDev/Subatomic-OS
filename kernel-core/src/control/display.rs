use crate::requests;
use crate::sync::init::InitData;
use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::Pixel;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Size};
use embedded_graphics::primitives::Rectangle;

/// Global display instance for writing graphics data to the framebuffer.
pub static DISPLAY: InitData<Display> = InitData::uninit();

/// The display to draw on.
///
/// Directly connected to the framebuffer provided by limine.
pub struct Display {
    width: usize,
    height: usize,
    pitch: usize,
    fb: &'static mut [u8],
    backbuffer: Vec<u8>,
}

impl Display {
    /// Create a new display instance.
    pub fn new() -> Self {
        // TODO: config without display?
        let fb = *requests::framebuffer()
            .framebuffers()
            .iter()
            .next()
            .expect("No display found.");

        let pitch = fb.pitch as usize;
        let height = fb.height as usize;
        let width = fb.width as usize;
        let fb_size = pitch * height;

        let slice = unsafe { core::slice::from_raw_parts_mut(fb.address() as *mut u8, fb_size) };

        Self {
            width,
            height,
            pitch,
            fb: slice,
            backbuffer: vec![0; fb_size],
        }
    }

    /// Get the framebuffer width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the framebuffer height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Set the pixel at `x` and `y` to the given color.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = y * self.pitch + x * 4;
        self.backbuffer[offset] = color.b();
        self.backbuffer[offset + 1] = color.g();
        self.backbuffer[offset + 2] = color.r();
        self.backbuffer[offset + 3] = 0;
    }

    /// Flushes the RAM backbuffer to VRAM in a single memory block copy.
    pub fn flush(&mut self) {
        self.fb.copy_from_slice(&self.backbuffer);
    }
}

impl Dimensions for Display {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::zero(),
            Size::new(self.width as u32, self.height as u32),
        )
    }
}

impl DrawTarget for Display {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            self.set_pixel(pixel.0.x as usize, pixel.0.y as usize, pixel.1);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let intersection = area.intersection(&self.bounding_box());
        if intersection.is_zero_sized() {
            return Ok(());
        }

        let pixel_bytes = [color.b(), color.g(), color.r(), 0];
        let x_start = intersection.top_left.x as usize;
        let y_start = intersection.top_left.y as usize;
        let width = intersection.size.width as usize;
        let height = intersection.size.height as usize;

        for y in y_start..(y_start + height) {
            let row_offset = y * self.pitch + x_start * 4;
            for x in 0..width {
                let offset = row_offset + x * 4;
                self.backbuffer[offset..offset + 4].copy_from_slice(&pixel_bytes);
            }
        }

        Ok(())
    }
}
