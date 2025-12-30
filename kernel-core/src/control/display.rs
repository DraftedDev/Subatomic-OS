use crate::requests;
use crate::sync::init::InitData;
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
    buf: Vec<u8>,
    fb: &'static mut [u8],
}

impl Display {
    /// Create a new display instance.
    pub fn new() -> Self {
        // TODO: config without display?
        let fb = unsafe { requests::framebuffer() }
            .framebuffers()
            .next()
            .expect("No display found.");

        let slice = unsafe {
            core::slice::from_raw_parts_mut(fb.addr(), fb.pitch() as usize * fb.height() as usize)
        };

        let mut buf = Vec::with_capacity(slice.len());

        unsafe {
            buf.set_len(slice.len());
        }

        buf.copy_from_slice(slice);

        Self {
            width: fb.width() as usize,
            height: fb.height() as usize,
            buf,
            fb: slice,
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

    /// Present the back-buffer to the framebuffer.
    pub fn present(&mut self) {
        self.fb.copy_from_slice(&self.buf);
    }

    /// Set the pixel at `x` and `y` to the given color.
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        if x >= self.width || y >= self.height {
            return;
        }

        let pitch = self.width * 4;
        let offset = y * pitch + x * 4;

        self.buf[offset] = color.b();
        self.buf[offset + 1] = color.g();
        self.buf[offset + 2] = color.r();
        self.buf[offset + 3] = 0;
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
}
