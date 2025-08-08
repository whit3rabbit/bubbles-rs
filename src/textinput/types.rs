//! Core types for the textinput component.

use bubbletea_rs::Msg;

/// Internal messages for clipboard operations.
/// Matches Go's pasteMsg and pasteErrMsg types.
/// Clipboard paste message carrying raw text.
#[derive(Debug, Clone)]
pub struct PasteMsg(pub String);

/// Clipboard paste error message.
#[derive(Debug, Clone)]
pub struct PasteErrMsg(pub String);

/// EchoMode sets the input behavior of the text input field.
/// Matches Go's EchoMode enum exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EchoMode {
    /// EchoNormal displays text as is. This is the default behavior.
    EchoNormal,
    /// EchoPassword displays the EchoCharacter mask instead of actual characters.
    /// This is commonly used for password fields.
    EchoPassword,
    /// EchoNone displays nothing as characters are entered. This is commonly
    /// seen for password fields on the command line.
    EchoNone,
}

/// ValidateFunc is a function that returns an error if the input is invalid.
/// Add Send to satisfy bubbletea-rs Model:Send bound transitively.
pub type ValidateFunc = Box<dyn Fn(&str) -> Result<(), String> + Send>;

impl From<PasteMsg> for Msg {
    fn from(msg: PasteMsg) -> Self {
        Box::new(msg) as Msg
    }
}

impl From<PasteErrMsg> for Msg {
    fn from(msg: PasteErrMsg) -> Self {
        Box::new(msg) as Msg
    }
}
