use alloc::vec::Vec;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::text::Text;
pub use ustyle;
use ustyle::Span;

/// A text box widget for [embedded_graphics].
pub struct TextBox<'a> {
    buf: &'a str,
    style: MonoTextStyle<'a, Rgb888>,
}

impl<'a> TextBox<'a> {
    const PARSE_CAPACITY: usize = 4;
    const LEFT_PADDING: i32 = 5;

    /// Create a new text box.
    pub fn new(buf: &'a str, style: MonoTextStyle<'a, Rgb888>) -> Self {
        Self { buf, style }
    }
}

impl<'a> Drawable for TextBox<'a> {
    type Color = Rgb888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        // TODO: make this faster somehow
        let buf = self.buf.replace("\t", "   ");
        target.clear(self.style.background_color.unwrap())?;

        let char_size = self.style.font.character_size;
        let line_height = char_size.height as i32;

        // Calculate how many lines fit in the display
        let max_lines = target.bounding_box().size.height as i32 / line_height;
        let mut all_lines: Vec<&str> = buf.lines().collect();

        // Only keep the last `max_lines`
        if all_lines.len() as i32 > max_lines {
            all_lines = all_lines.split_off(all_lines.len() - max_lines as usize);
        }

        let mut cursor = Point::new(Self::LEFT_PADDING, line_height);

        for line in all_lines {
            let spans =
                Span::decode_capacity(line, Self::PARSE_CAPACITY).expect("Failed to decode spans");

            for span in spans {
                let mut style = self.style;

                style.text_color = span
                    .style
                    .foreground
                    .to_rgb()
                    .map(|(r, g, b)| Rgb888::new(r, g, b))
                    .or(self.style.text_color);

                style.background_color = span
                    .style
                    .background
                    .to_rgb()
                    .map(|(r, g, b)| Rgb888::new(r, g, b));

                // Draw span text
                Text::new(&span.text, cursor, style).draw(target)?;
                cursor.x += span.text.len() as i32 * char_size.width as i32;
            }

            // Move to next line
            cursor.x = Self::LEFT_PADDING;
            cursor.y += line_height;
        }

        Ok(())
    }
}
