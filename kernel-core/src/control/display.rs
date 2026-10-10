use crate::requests;
use crate::sync::init::InitData;
use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use embedded_graphics::Pixel;
use embedded_graphics::geometry::Dimensions;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::prelude::{DrawTarget, Point, Size};
use embedded_graphics::primitives::Rectangle;

/// Global display instance for writing graphics data to the framebuffer.
pub static DISPLAY: InitData<Box<dyn Display>> = InitData::uninit();

/// A general display abstraction.
pub trait Display: Send + Sync + 'static {
    /// Get the width of the display.
    fn width(&self) -> u32;
    /// Get the height of the display.
    fn height(&self) -> u32;
    /// Get the stride/pitch of the display.
    fn stride(&self) -> u32;
    /// Get display pixel format.
    fn format(&self) -> PixelFormat;
    /// Get the display framebuffer.
    fn buffer(&mut self) -> &mut [u8];
    /// Flush the framebuffer to the actual display.
    fn flush(&mut self);
}

/// The display pixel format.
pub enum PixelFormat {
    /// 8x4 blue-green-red-alpha format.
    Bgra8888,
    /// 8x4 red-green-blue-alpha format.
    Rgba8888,
    /// 5-6-5 red-blue-green format.
    Rgb565,
}

/// The display to draw on.
///
/// Directly connected to the framebuffer provided by limine.
pub struct BootDisplay {
    width: u32,
    height: u32,
    pitch: u32,
    fb: &'static mut [u8],
    backbuffer: Vec<u8>,
}

impl BootDisplay {
    /// Create a new display instance.
    pub fn new() -> Self {
        // TODO: config without display?
        let fb = *requests::framebuffer()
            .framebuffers()
            .iter()
            .next()
            .expect("No display found.");

        let pitch = fb.pitch as u32;
        let height = fb.height as u32;
        let width = fb.width as u32;
        let fb_size = (pitch * height) as usize;

        let slice = unsafe { core::slice::from_raw_parts_mut(fb.address() as *mut u8, fb_size) };

        Self {
            width,
            height,
            pitch,
            fb: slice,
            backbuffer: vec![0; fb_size],
        }
    }
}

impl Display for BootDisplay {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn stride(&self) -> u32 {
        self.pitch
    }

    fn format(&self) -> PixelFormat {
        PixelFormat::Rgba8888
    }

    fn buffer(&mut self) -> &mut [u8] {
        self.backbuffer.as_mut()
    }

    fn flush(&mut self) {
        self.fb.copy_from_slice(&self.backbuffer);
    }
}

impl Dimensions for Box<dyn Display> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::zero(), Size::new(self.width(), self.height()))
    }
}

impl DrawTarget for Box<dyn Display> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let stride = self.stride() as usize;
        let w = self.width();
        let h = self.height();
        let buf = self.buffer();

        for Pixel(Point { x, y }, color) in pixels {
            if x >= 0 && x < w as i32 && y >= 0 && y < h as i32 {
                let idx = (y as usize * stride) + (x as usize * 4);
                buf[idx] = color.b();
                buf[idx + 1] = color.g();
                buf[idx + 2] = color.r();
                buf[idx + 3] = 0xFF;
            }
        }

        Ok(())
    }
}
