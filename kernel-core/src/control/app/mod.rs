use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use pc_keyboard::DecodedKey;
use ratatui::Frame;

/// Contains the [breakout::BreakoutApp].
pub mod breakout;

/// An application run inside the [Control].
pub trait App: Send + Sync + 'static {
    /// Render the app to the control frame.
    fn render(&mut self, frame: &mut Frame) -> AppCommand;
    /// Handle user input.
    fn handle_input(&mut self, key: DecodedKey) -> AppCommand;
    /// Exits the app.
    fn exit(&mut self);
}

/// The command returned by [App] operations.
pub enum AppCommand {
    /// Set the control app.
    SetApp(Box<dyn App>),
    /// Exit the app and optionally print the given message.
    Exit(Option<String>),
    /// Execute multiple app commands.
    Multiple(Vec<AppCommand>),
    /// Continue the app execution.
    Continue,
}
