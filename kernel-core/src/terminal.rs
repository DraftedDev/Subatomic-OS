use crate::control::InnerControl;
use alloc::vec::Vec;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color as TuiColor, Style as TuiStyle};
use ratatui::widgets::Widget;
use ustyle::{Attributes, Color, Style};

/// A terminal-style text box widget.
pub struct TerminalBox<'a> {
    buf: Vec<Vec<(char, Style)>>,
    command: &'a str,
    default: Style,
    scroll_offset: usize,
}

impl<'a> TerminalBox<'a> {
    /// Create a new [TerminalBox] from a buffer, command line and default style.
    pub fn new(
        buf: Vec<Vec<(char, Style)>>,
        command: &'a str,
        default: Style,
        scroll_offset: usize,
    ) -> Self {
        Self {
            buf,
            command,
            default,
            scroll_offset,
        }
    }

    #[inline]
    fn style_to_tui(style: &Style, default: &Style) -> TuiStyle {
        let fg = style.foreground.to_rgb().unwrap_or(
            default
                .foreground
                .to_rgb()
                .expect("Default foreground cannot be `None`"),
        );

        let bg = style.background.to_rgb().unwrap_or(
            default
                .background
                .to_rgb()
                .expect("Default foreground cannot be `None`"),
        );

        TuiStyle::new()
            .fg(TuiColor::from(fg))
            .bg(TuiColor::from(bg))
    }
}

impl<'a> Widget for TerminalBox<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.reset();

        // Build visible lines only
        let max_lines = area.height as usize;

        // Append command line
        let cmd = {
            let mut buf = Vec::with_capacity(
                self.command.len() + 3, // prefix + suffix + space
            );

            buf.push((InnerControl::COMMAND_PREFIX, self.default));
            buf.push((' ', self.default));

            for ch in self.command.chars() {
                buf.push((ch, self.default));
            }

            buf.push((
                InnerControl::COMMAND_SUFFIX,
                Style::new(
                    Color::BrighterGray,
                    Color::BrighterGray,
                    Attributes::empty(),
                ),
            ));

            buf
        };

        self.buf.push(cmd);

        let max_scroll = self.buf.len().saturating_sub(max_lines);
        let scroll = self.scroll_offset.min(max_scroll);
        let start = max_scroll.saturating_sub(scroll);

        for (row, line) in self.buf[start..].iter().enumerate() {
            let mut y = area.y + row as u16;
            let mut x = area.x;

            for (ch, style) in line {
                if x >= area.right() {
                    x = area.x;
                    y += 1;
                    if y >= area.bottom() {
                        break;
                    }
                }

                let cell = buf.cell_mut((x, y)).unwrap();
                cell.set_char(*ch);
                cell.set_style(Self::style_to_tui(style, &self.default));

                x += 1;
            }
        }
    }
}
