use crate::collections::FastMap;
use crate::control::command::{Command, builtin};
use crate::control::display::{DISPLAY, Display};
use crate::control::input::{INPUT, InputControl};
use crate::serial;
use crate::sync::init::InitData;
use crate::sync::mutex::Mutex;
use crate::terminal::TerminalBox;
use crate::wrapper::SendSyncWrapper;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use core::fmt::Write;
use crossbeam_queue::SegQueue;
use embedded_graphics::pixelcolor::Rgb888;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, TerminalAlignment};
use pc_keyboard::{DecodedKey, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders};
use ustyle::{Attributes, Color, Style};

/// Contains the [Display] type.
pub mod display;

/// Contains the [InputControl] struct.
pub mod input;

/// Contains the [Command] struct and related features.
pub mod command;

/// Global [Control] instance.
pub static CONTROL: InitData<Control> = InitData::uninit();

static mut INIT: bool = false;

/// Initialize the global [InputControl] and the [Control] instances.
pub unsafe fn init() {
    unsafe {
        INPUT.init(InputControl::new());
        CONTROL.init(Control::new());

        INIT = true;
    }
}

/// Get if the control is initialized.
pub fn is_init() -> bool {
    unsafe { INIT }
}

/// Structure for controlling the entire kernel using user input.
///
/// Similar to the shell in Linux, but always active and globally reachable.
pub struct Control {
    queue: SegQueue<String>,
    registry: FastMap<&'static str, Command>,
    inner: Mutex<InnerControl>,
}

impl Control {
    const MAX_EXECUTED_COMMANDS: u8 = 4;

    /// Create a new control instance.
    pub unsafe fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            registry: FastMap::from_iter(
                builtin::COMMANDS
                    .into_iter()
                    .map(|command| (command.name, *command)),
            ),
            inner: Mutex::new(unsafe { InnerControl::new() }),
        }
    }

    /// Update the control.
    pub fn update(&self) {
        self.run(|inner| {
            inner.handle_input(&self.queue);
            inner.render();
        });

        self.execute(Self::MAX_EXECUTED_COMMANDS)
            .unwrap_or_else(|err| log::error!("{err}"));
    }

    /// Lock the [InnerControl] and run the specified closure on it, then unlock it at last.
    pub fn run(&self, func: impl FnOnce(&mut InnerControl)) {
        self.inner.run(func)
    }

    /// Execute all the commands in queue, but a maximum of `max` times.
    pub fn execute(&self, max: u8) -> Result<(), String> {
        let mut i = 0;

        while let Some(query) = self.queue.pop()
            && i <= max
        {
            let (name, args) = query.trim().split_once(' ').unwrap_or((query.as_str(), ""));

            match name {
                "help" => self.log_help(),
                "" => (),
                _ => {
                    let command = self.registry.get(name.trim()).ok_or_else(|| {
                        format!("Command '{query}' not found! Type 'help' for help.")
                    })?;

                    (command.run)(args.trim().to_string())?;
                }
            }

            i += 1;
        }

        Ok(())
    }

    /// Log the help message to the control.
    pub fn log_help(&self) {
        let mut help = String::with_capacity(self.registry.len() * 32);

        for command in self.registry.values() {
            help.push_str(&format!(
                "\n{}:\n\
\tDescription: {}\n\
\tUsage: {}\n",
                command.name, command.description, command.usage
            ));
        }

        log::info!("Command-Line Help:\n{help}");
    }
}

/// The inner control value of the actual [Control].
///
/// This provides functionality that requires mutability,
/// therefore it's locked behind [Control::run].
pub struct InnerControl {
    terminal: SendSyncWrapper<Terminal<EmbeddedBackend<'static, Display, Rgb888>>>,
    buf: String,
    command: String,
    scroll_offset: usize,
}

impl InnerControl {
    /// The command prefix.
    pub const COMMAND_PREFIX: &'static str = "> ";
    /// The command suffix.
    pub const COMMAND_SUFFIX: &'static str = "|";
    const STRING_BUF_CAPACITY: usize = 1024;
    const COMMAND_BUF_CAPACITY: usize = 16;
    const EXPANDED_TAB: &'static str = "    ";
    const BACKGROUND: Color = Color::DarkerGray;
    const FOREGROUND: Color = Color::BrighterGray;

    /// Create a new control instance.
    pub unsafe fn new() -> Self {
        unsafe {
            Self {
                terminal: SendSyncWrapper::new(
                    Terminal::new(EmbeddedBackend::new(
                        DISPLAY.get_mut(),
                        EmbeddedBackendConfig {
                            flush_callback: Box::new(|_| ()),
                            font_regular: mousefood::fonts::MONO_9X18,
                            font_bold: Some(mousefood::fonts::MONO_9X18_BOLD),
                            font_italic: None,
                            vertical_alignment: TerminalAlignment::Start,
                            horizontal_alignment: TerminalAlignment::Start,
                        },
                    ))
                    .expect("Failed to build terminal"),
                ),
                buf: String::with_capacity(Self::STRING_BUF_CAPACITY),
                command: String::with_capacity(Self::COMMAND_BUF_CAPACITY),
                scroll_offset: 0,
            }
        }
    }

    fn handle_input(&mut self, queue: &SegQueue<String>) {
        let input = INPUT.get();
        while let Some(key) = input.pop() {
            match key {
                DecodedKey::Unicode(ch) => match ch {
                    // New line => execute
                    '\n' => {
                        let command: String = self.command.drain(..).collect();

                        self.buf.push_str(Self::COMMAND_PREFIX);
                        self.buf.push_str(&command);
                        self.buf.push('\n');

                        queue.push(command);
                    }

                    // Backspace => delete last character
                    '\x08' => {
                        self.command.pop();
                    }

                    // Else => push to command
                    _ => self.command.push(ch),
                },
                DecodedKey::RawKey(code) => match code {
                    // Scroll up => increment scroll offset
                    KeyCode::ArrowUp => self.scroll_offset = self.scroll_offset.saturating_add(1),

                    // Scroll down => decrement scroll offset
                    KeyCode::ArrowDown => self.scroll_offset = self.scroll_offset.saturating_sub(1),

                    // Else => do nothing
                    _ => (),
                },
                _ => (),
            }
        }
    }

    fn render(&mut self) {
        let screen = Rect::from(
            self.terminal
                .backend()
                .size()
                .expect("Failed to get backend size"),
        );

        self.terminal
            .draw(|frame| {
                let block = Block::new()
                    .borders(Borders::all())
                    .border_type(BorderType::Rounded);

                let inner = block.inner(screen);

                frame.render_widget(block, screen);

                frame.render_widget(
                    TerminalBox::new(
                        &self.buf,
                        &self.command,
                        Style::new(Self::FOREGROUND, Self::BACKGROUND, Attributes::empty()),
                        self.scroll_offset,
                    ),
                    inner,
                );
            })
            .expect("Failed to draw terminal");
    }
}

impl Write for InnerControl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for ch in s.chars() {
            if ch == '\t' {
                self.buf.push_str(Self::EXPANDED_TAB);
            } else {
                self.buf.push(ch);
            }
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> core::fmt::Result {
        if c == '\t' {
            self.buf.push_str(InnerControl::EXPANDED_TAB);
        } else {
            self.buf.push(c);
        }

        Ok(())
    }
}
