use crate::control::InnerControl;
use alloc::vec::Vec;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color as TuiColor, Style as TuiStyle};
use ratatui::widgets::Widget;
use ustyle::{Attributes, Color, Style};

/// A terminal-style text box widget.
pub struct TerminalBox<'a> {
    buf: &'a [Vec<(char, Style)>],
    command: &'a str,
    default: Style,
    scroll_offset: usize,
}

impl<'a> TerminalBox<'a> {
    /// Create a new [TerminalBox] from a buffer, command line and default style.
    pub fn new(
        buf: &'a [Vec<(char, Style)>],
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

    fn build_command_line(command: &str, default: Style) -> Vec<(char, Style)> {
        // TODO: Don't create extra Vec for command line
        let mut buf = Vec::with_capacity(command.len() + 3);

        buf.push((InnerControl::COMMAND_PREFIX, default));
        buf.push((' ', default));

        for ch in command.chars() {
            buf.push((ch, default));
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
    }
}

impl<'a> Widget for TerminalBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.reset();

        let max_lines = area.height as usize;
        let total_lines = self.buf.len() + 1; // + command line

        let max_scroll = total_lines.saturating_sub(max_lines);
        let scroll = self.scroll_offset.min(max_scroll);
        let start = max_scroll.saturating_sub(scroll);

        for row in 0..max_lines {
            let line_idx = start + row;
            if line_idx >= total_lines {
                break;
            }

            let y = area.y + row as u16;
            let mut x = area.x;

            let line: &[(char, Style)];

            // Normal buffer line
            let cmd_storage;
            if line_idx < self.buf.len() {
                line = &self.buf[line_idx];
            } else {
                // Command line (built on the fly, no cloning of scrollback)
                cmd_storage = Self::build_command_line(self.command, self.default);
                line = &cmd_storage;
            }

            for (ch, style) in line {
                if x >= area.right() {
                    break;
                }

                let cell = buf.cell_mut((x, y)).unwrap();
                cell.set_char(*ch);
                cell.set_style(Self::style_to_tui(style, &self.default));

                x += 1;
            }
        }
    }
}
