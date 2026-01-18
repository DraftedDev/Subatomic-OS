use crate::control::InnerControl;
use alloc::vec::Vec;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color as TuiColor, Style as TuiStyle};
use ratatui::widgets::Widget;
use ustyle::{Attributes, Color, Span, Style};

/// A terminal-style text box widget.
pub struct TerminalBox<'a> {
    buf: &'a [Span],
    command: &'a str,
    default: Style,
    scroll_offset: usize,
}

impl<'a> TerminalBox<'a> {
    /// Create a new [TerminalBox] from a buffer, command line and default style.
    pub fn new(buf: &'a [Span], command: &'a str, default: Style, scroll_offset: usize) -> Self {
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
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear region
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.cell_mut((x, y)).expect("Failed to get cell").reset();
            }
        }

        // Build visible lines only
        let max_lines = area.height as usize;
        let max_width = area.width as usize;

        let mut lines: Vec<Vec<(char, Style)>> = Vec::new();
        let mut current = Vec::with_capacity(max_width);

        for span in self.buf {
            for ch in span.text.chars() {
                if ch == '\n' {
                    lines.push(core::mem::take(&mut current));
                } else {
                    current.push((ch, span.style));

                    if current.len() == max_width {
                        lines.push(core::mem::take(&mut current));
                    }
                }
            }
        }

        if !current.is_empty() {
            lines.push(current);
        }

        // Append command line
        let mut cmd = Vec::with_capacity(
            InnerControl::COMMAND_PREFIX.len()
                + self.command.len()
                + InnerControl::COMMAND_SUFFIX.len(),
        );

        for ch in InnerControl::COMMAND_PREFIX.chars() {
            cmd.push((ch, self.default));
        }

        for ch in self.command.chars() {
            cmd.push((ch, self.default));
        }

        for ch in InnerControl::COMMAND_SUFFIX.chars() {
            cmd.push((
                ch,
                Style::new(
                    Color::BrighterGray,
                    Color::BrighterGray,
                    Attributes::empty(),
                ),
            ));
        }

        lines.push(cmd);

        let max_scroll = lines.len().saturating_sub(max_lines);
        let scroll = self.scroll_offset.min(max_scroll);
        let start = max_scroll.saturating_sub(scroll);

        for (row, line) in lines[start..].iter().enumerate() {
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

                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(*ch);
                    cell.set_style(Self::style_to_tui(style, &self.default));
                }

                x += 1;
            }
        }
    }
}
