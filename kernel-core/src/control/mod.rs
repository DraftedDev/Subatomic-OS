use crate::collections::FastMap;
use crate::control::app::{App, AppCommand};
use crate::control::command::{Command, builtin};
use crate::control::display::{DISPLAY, Display};
use crate::control::input::{INPUT, InputControl, ModifierFlags};
use crate::sync::init::InitData;
use crate::sync::mutex::Mutex;
use crate::terminal::TerminalBox;
use crate::wrapper::SendSyncWrapper;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Write;
use crossbeam_queue::SegQueue;
use embedded_graphics::pixelcolor::Rgb888;
use mousefood::{EmbeddedBackend, EmbeddedBackendConfig, TerminalAlignment};
use pc_keyboard::{DecodedKey, KeyCode};
use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, BorderType, Borders};
use ustyle::{Attributes, Color, Span, Style};

/// Contains the [Display] type.
pub mod display;

/// Contains the [InputControl] struct.
pub mod input;

/// Provides control application structures.
pub mod app;

/// Contains the [Command] struct and related features.
pub mod command;

/// Global [Control] instance.
pub static CONTROL: InitData<Control> = InitData::uninit();

static mut INIT: bool = false;

/// Initialize the global [InputControl] and the [Control] instances.
///
/// # Safety
/// This must only be called once, before any global [CONTROL] use.
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
    ///
    /// # Safety
    ///
    /// See [InnerControl::new].
    pub unsafe fn new() -> Self {
        Self {
            queue: SegQueue::new(),
            registry: FastMap::from_iter(
                builtin::COMMANDS
                    .iter()
                    .map(|command| (command.name, *command)),
            ),
            inner: Mutex::new(unsafe { InnerControl::new() }),
        }
    }

    /// Registers a command for the control.
    ///
    /// Can only be used, by obtaining an **unsafe** mutable reference to the global [CONTROL].
    pub fn register(&mut self, command: Command) {
        self.registry.insert(command.name, command);
    }

    /// Update the control.
    pub fn update(&self) {
        self.run(|inner| {
            if inner.is_dirty() {
                inner.handle_input(&self.queue);
                inner.render();

                inner.set_dirty(false);
            }
        });

        self.execute(Self::MAX_EXECUTED_COMMANDS)
            .unwrap_or_else(|err| log::error!("{err}"));
    }

    /// Lock the [InnerControl] and run the specified closure on it, then unlock it at last.
    pub fn run<R>(&self, func: impl FnOnce(&mut InnerControl) -> R) -> R {
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
        const HELP_START: &str = "Control Help:\n\n\
            This is the control, the main interface to the kernel.\n\
            You can think of this as an overarching root shell.\n\
            Use ↑ and ↓ to scroll through the command history.\n\
            Use Ctrl + ↑ and Ctrl + ↓ to scroll through the terminal.\n\
            Available Commands:\n\n";

        let mut help = String::with_capacity(self.registry.len() * 32 + HELP_START.len());

        help.push_str(HELP_START);

        for command in self.registry.values() {
            help.push_str(&format!(
                "{}:\n\
\tDescription: {}\n\
\tUsage: {}\n",
                command.name, command.description, command.usage
            ));
        }

        log::info!("{help}");
    }
}

/// The inner control value of the actual [Control].
///
/// This provides functionality that requires mutability,
/// therefore it's locked behind [Control::run].
pub struct InnerControl {
    terminal: SendSyncWrapper<Terminal<EmbeddedBackend<'static, Box<dyn Display>, Rgb888>>>,
    lines: Vec<Vec<(char, Style)>>,
    string_buf: String,
    command: String,
    scroll_offset: usize,
    max_width: usize,
    app: Option<Box<dyn App>>,
    dirty: bool,
    history: VecDeque<String>,
    history_index: Option<usize>,
    saved_draft: String,
}

impl InnerControl {
    /// The command prefix.
    pub const COMMAND_PREFIX: char = '>';
    /// The command suffix.
    pub const COMMAND_SUFFIX: char = '|';

    const LINES_CAPACITY: usize = 128;
    const STRING_BUF_CAPACITY: usize = 256;
    const PARSE_CAPACITY: usize = 4;
    const COMMAND_BUF_CAPACITY: usize = 16;
    const HISTORY_CAPACITY: usize = 16;
    const EXPANDED_TAB: &'static str = "    ";
    const BACKGROUND: Color = Color::DarkerGray;
    const FOREGROUND: Color = Color::BrighterGray;

    /// Create a new control instance.
    ///
    /// # Safety
    /// This should only be called once, since it mutably uses the [DISPLAY] global.
    pub unsafe fn new() -> Self {
        let terminal = unsafe {
            SendSyncWrapper::new(
                Terminal::new(EmbeddedBackend::new(
                    DISPLAY.get_mut(),
                    EmbeddedBackendConfig {
                        flush_callback: Box::new(|display| {
                            display.flush();
                        }),
                        font_regular: mousefood::fonts::MONO_9X18,
                        font_bold: Some(mousefood::fonts::MONO_9X18_BOLD),
                        font_italic: None,
                        vertical_alignment: TerminalAlignment::Start,
                        horizontal_alignment: TerminalAlignment::Start,
                    },
                ))
                .expect("Failed to build terminal"),
            )
        };

        let max_width = terminal.size().unwrap().width as usize;

        Self {
            terminal,
            lines: Vec::with_capacity(Self::LINES_CAPACITY),
            string_buf: String::with_capacity(Self::STRING_BUF_CAPACITY),
            command: String::with_capacity(Self::COMMAND_BUF_CAPACITY),
            scroll_offset: 0,
            max_width,
            app: None,
            dirty: false,
            history: VecDeque::with_capacity(Self::HISTORY_CAPACITY),
            history_index: None,
            saved_draft: String::with_capacity(Self::COMMAND_BUF_CAPACITY),
        }
    }

    /// Returns whether the control state is dirty.
    pub fn is_dirty(&self) -> bool {
        self.dirty || !INPUT.get().is_empty() || self.app.is_some()
    }

    /// Sets the dirty state of the control.
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// Set the current control [App].
    ///
    /// If there already was another app active, it will be exited.
    pub fn set_app(&mut self, app: impl App) {
        if let Some(mut app) = self.app.replace(Box::new(app)) {
            app.exit();
        }

        self.set_dirty(true);
    }

    /// Adds a command to history and resets history navigation state.
    pub fn add_to_history(&mut self, command: String) {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return;
        }

        if self.history.front().map(|s| s.as_str()) != Some(trimmed) {
            if self.history.len() == Self::HISTORY_CAPACITY {
                self.history.pop_back();
            }
            self.history.push_front(command);
        }

        self.history_index = None;
        self.saved_draft.clear();
    }

    fn handle_input(&mut self, queue: &SegQueue<String>) {
        let input = INPUT.get();

        let mods = input.modifiers();

        while let Some(key) = input.pop() {
            let mut command = AppCommand::Continue;

            if let Some(app) = &mut self.app {
                command = app.handle_input(key);
            } else {
                match key {
                    DecodedKey::Unicode(ch) => match ch {
                        // New line => execute
                        '\n' => {
                            let command: String = core::mem::take(&mut self.command);

                            self.string_buf
                                .push_str(&format!("{} {command}\n", Self::COMMAND_PREFIX));

                            self.add_to_history(command.clone());
                            queue.push(command);
                        }

                        // Backspace => delete last character
                        '\x08' => {
                            self.command.pop();
                            self.scroll_offset = 0;
                        }

                        // Else => push to command
                        _ => {
                            self.command.push(ch);
                            self.scroll_offset = 0;
                        }
                    },

                    DecodedKey::RawKey(code) => match code {
                        KeyCode::ArrowUp => {
                            if mods.intersects(ModifierFlags::L_CTRL | ModifierFlags::R_CTRL) {
                                self.scroll_offset = self.scroll_offset.saturating_add(1);
                            } else {
                                self.navigate_history_up();
                            }
                        }

                        KeyCode::ArrowDown => {
                            if mods.intersects(ModifierFlags::L_CTRL | ModifierFlags::R_CTRL) {
                                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                            } else {
                                self.navigate_history_down();
                            }
                        }

                        _ => (),
                    },
                }
            }

            self.handle_command(command);
        }
    }

    fn navigate_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.saved_draft = self.command.clone();
                self.history_index = Some(0);
                self.command = self.history[0].clone();
            }
            Some(idx) => {
                if idx + 1 < self.history.len() {
                    let next_idx = idx + 1;
                    self.history_index = Some(next_idx);
                    self.command = self.history[next_idx].clone();
                }
            }
        }
    }

    fn navigate_history_down(&mut self) {
        if let Some(idx) = self.history_index {
            if idx > 0 {
                let next_idx = idx - 1;
                self.history_index = Some(next_idx);
                self.command = self.history[next_idx].clone();
            } else {
                self.history_index = None;
                self.command = core::mem::take(&mut self.saved_draft);
            }
        }
    }

    fn render(&mut self) {
        let mut command = AppCommand::Continue;

        if let Some(app) = &mut self.app {
            self.terminal
                .draw(|frame| {
                    command = app.render(frame);
                })
                .expect("Failed to draw app");
        } else {
            self.render_terminal();
        }

        self.handle_command(command);
    }

    fn render_terminal(&mut self) {
        // Parse temp buffer if not empty
        if !self.string_buf.is_empty() {
            let string = core::mem::take(&mut self.string_buf);

            let spans = Span::decode_capacity(&string, Self::PARSE_CAPACITY)
                .expect("Failed to parse spans");

            let mut current = Vec::with_capacity(self.max_width);

            for span in spans {
                for ch in span.text.chars() {
                    if ch == '\n' {
                        self.lines.push(core::mem::take(&mut current));
                    } else {
                        current.push((ch, span.style));

                        if current.len() == self.max_width {
                            self.lines.push(core::mem::take(&mut current));
                        }
                    }
                }
            }

            if !current.is_empty() {
                self.lines.push(current);
            }
        }

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

                frame.render_widget(
                    TerminalBox::new(
                        &self.lines,
                        &self.command,
                        Style::new(Self::FOREGROUND, Self::BACKGROUND, Attributes::empty()),
                        self.scroll_offset,
                    ),
                    inner,
                );

                frame.render_widget(block, screen);
            })
            .expect("Failed to draw terminal");
    }

    fn handle_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::SetApp(app) => {
                self.app.replace(app).unwrap().exit();
            }

            AppCommand::Exit(msg) => {
                self.app.take().unwrap().exit();

                if let Some(msg) = msg {
                    self.string_buf.push_str(&msg);
                }
            }

            AppCommand::Multiple(commands) => {
                for command in commands {
                    self.handle_command(command);
                }
            }

            AppCommand::Continue => (),
        }
    }
}

impl Write for InnerControl {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.set_dirty(true);

        for ch in s.chars() {
            if ch == '\t' {
                self.string_buf.push_str(Self::EXPANDED_TAB);
            } else {
                self.string_buf.push(ch);
            }
        }

        Ok(())
    }
}
