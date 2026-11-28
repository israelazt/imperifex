//! Pasting from the clipboard and copying to it.

use imperifex_domain::i18n::{fill, Text};
use imperifex_domain::log_warn;
use imperifex_domain::paste::{for_editor, for_program, Paste};
use imperifex_domain::ports::Clipboard;

use super::{redraw_if, Core, WindowCommand};

#[derive(Default)]
pub(super) struct Pasting {
    clipboard: Option<Box<dyn Clipboard>>,
    /// The text the user was asked about, and its lines.
    asked: Option<(String, usize)>,
}

impl Pasting {
    /// Any other thing done after being asked is a no.
    pub fn forget(&mut self) -> bool {
        self.asked.take().is_some()
    }
}

impl Core {
    pub fn use_clipboard(&mut self, clipboard: Box<dyn Clipboard>) {
        self.pasting.clipboard = Some(clipboard);
    }

    /// What the status bar warns of.
    pub(super) fn warning(&self) -> Option<String> {
        let (_, lines) = self.pasting.asked.as_ref()?;
        Some(fill(self.language, Text::PasteLines, &[lines]))
    }

    /// Writes the clipboard in the editor, or else sends it to the program.
    pub fn paste(&mut self) -> Vec<WindowCommand> {
        let Some(clipboard) = &self.pasting.clipboard else {
            return Vec::new();
        };
        let text = match clipboard.read() {
            Ok(Some(text)) => text,
            Ok(None) => return Vec::new(),
            Err(error) => {
                log_warn!("nothing was pasted: {error}");
                return Vec::new();
            }
        };
        let asked = self.pasting.asked.take();
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        if self.typing.prompt(tab).is_some() {
            tab.to_screen();
            return redraw_if(self.typing.paste(&for_editor(&text)));
        }
        let again = asked.as_ref().is_some_and(|(before, _)| *before == text);
        match for_program(&text, tab.terminal().modes, again) {
            Paste::Nothing => redraw_if(asked.is_some()),
            Paste::Ask(lines) => {
                self.pasting.asked = Some((text, lines));
                redraw_if(true)
            }
            Paste::Send(bytes) => {
                if let Err(error) = tab.send(&bytes) {
                    log_warn!("the paste did not reach the shell: {error}");
                }
                redraw_if(true)
            }
        }
    }
}
