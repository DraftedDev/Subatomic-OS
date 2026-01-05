use crate::control::InnerControl;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};
use embedded_graphics::text::Text;
use embedded_graphics::text::renderer::CharacterStyle;
pub use ustyle;
use ustyle::Span;

/// A terminal-style text box widget for [embedded_graphics].
pub struct TerminalBox<'a> {
    buf: &'a str,
    command: &'a str,
    style: MonoTextStyle<'a, Rgb888>,
}

impl<'a> TerminalBox<'a> {
    const PARSE_CAPACITY: usize = 4;
    const LEFT_PADDING: i32 = 5;

    /// Create a new terminal box.
    pub fn new(buf: &'a str, command: &'a str, style: MonoTextStyle<'a, Rgb888>) -> Self {
        Self {
            buf,
            command,
            style,
        }
    }
}

impl<'a> Drawable for TerminalBox<'a> {
    type Color = Rgb888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.clear(self.style.background_color.unwrap())?;

        let char_size = self.style.font.character_size;
        let line_height = char_size.height as i32;
        let char_width = char_size.width as i32;

        let max_lines = target.bounding_box().size.height as i32 / line_height - 1;

        let mut cursor = Point::new(Self::LEFT_PADDING, line_height);

        // Skip directly to the last `max_lines`
        let total_lines = self.buf.lines().count() as i32;
        let skip = (total_lines - max_lines).max(0) as usize;

        for line in self.buf.lines().skip(skip) {
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

                Text::new(&span.text, cursor, style).draw(target)?;

                // Monospaced: byte length == glyph count for ASCII
                cursor.x += span.text.len() as i32 * char_width;
            }

            cursor.x = Self::LEFT_PADDING;
            cursor.y += line_height;
        }

        // Command line
        {
            // Prefix
            Text::new(InnerControl::COMMAND_PREFIX, cursor, self.style).draw(target)?;
            cursor.x += InnerControl::COMMAND_PREFIX.len() as i32 * char_width;

            // Command
            Text::new(self.command, cursor, self.style).draw(target)?;
            cursor.x += self.command.len() as i32 * char_width;

            // Suffix (cursor)
            let mut suffix_style = self.style;
            suffix_style.set_background_color(Some(Rgb888::WHITE));

            Text::new(InnerControl::COMMAND_SUFFIX, cursor, suffix_style).draw(target)?;
        }

        Ok(())
    }
}
