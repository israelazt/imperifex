//! The command editor of the window: while it is shown, what it does with each
//! key, how a command reaches the shell and what is remembered of it.

use imperifex_domain::command_history::{is_kept, line, read};
use imperifex_domain::completion::{folder_of, options, Word};
use imperifex_domain::editor::{editor_key, Editor, Outcome};
use imperifex_domain::font::pixels_of_points;
use imperifex_domain::input::KeyInput;
use imperifex_domain::integration::submission;
use imperifex_domain::log_warn;
use imperifex_domain::ports::{CommandHistory, Folders};
use imperifex_domain::shell::profile_of;
use imperifex_domain::shell_state::ShellState;
use imperifex_domain::view::EditorView;

use super::{redraw_if, Core, WindowCommand};
use crate::shell_tab::ShellTab;

/// What `Ctrl`+`C` is to a shell.
const INTERRUPT: &[u8] = b"\x03";

#[derive(Default)]
pub(super) struct Typing {
    editor: Editor,
    /// Chosen in the settings, and turned on and off while the window is open.
    pub on: bool,
    store: Option<Box<dyn CommandHistory>>,
    /// Whose history it is.
    profile: String,
    /// The last command could not be sent whole.
    refused: bool,
    /// Where `Tab` looks for names.
    folders: Option<Box<dyn Folders>>,
}

impl Typing {
    pub fn new(on: bool) -> Self {
        Self {
            on,
            ..Self::default()
        }
    }

    /// Starts with the history of the profile whose shell `line` opened.
    pub fn start(&mut self, line: &str) {
        self.profile = profile_of(line).to_string();
        self.editor = Editor::new(self.remembered());
    }

    /// The commands of the profile's file, which is cut if it grew too long.
    /// A history that cannot be read is an empty one.
    fn remembered(&self) -> Vec<String> {
        let Some(store) = &self.store else {
            return Vec::new();
        };
        let text = store.load(&self.profile).unwrap_or_else(|error| {
            log_warn!("no history: {error}");
            None
        });
        let (commands, excess) = read(&text.unwrap_or_default());
        if excess {
            let cut: String = commands.iter().map(|command| line(command)).collect();
            if let Err(error) = store.replace(&self.profile, &cut) {
                log_warn!("the history was not cut: {error}");
            }
        }
        commands
    }

    /// Offers the editor the paths that `word` can be, looking from the
    /// shell's folder. `false` if there is none.
    fn complete(&mut self, word: &Word, current: Option<&str>) -> bool {
        let (Some(folders), Some(folder)) = (&self.folders, folder_of(word, current)) else {
            return false;
        };
        let entries = folders.entries(&folder).unwrap_or_else(|error| {
            log_warn!("nothing to complete: {error}");
            Vec::new()
        });
        self.editor.offer(word.start, options(word, &entries))
    }

    /// Writes a paste in the editor; `false` if there was nothing to write.
    pub fn paste(&mut self, text: &str) -> bool {
        self.editor.paste(text);
        self.refused = false;
        !text.is_empty()
    }

    /// Puts the cursor where the mouse was pressed.
    pub fn place_cursor(&mut self, line: usize, column: usize) {
        self.editor.move_to(line, column);
    }

    /// Writes `command` in the history if it is to be kept, and says if it was.
    fn remember(&self, command: &str) -> bool {
        let kept = is_kept(command, self.editor.last());
        if let (true, Some(store)) = (kept, &self.store) {
            if let Err(error) = store.append(&self.profile, &line(command)) {
                log_warn!("the command was not kept: {error}");
            }
        }
        kept
    }

    /// The prompt after which the editor takes the command, if it is to be
    /// shown: the shell of `tab` waits for one and no program has the screen
    /// or the mouse.
    pub fn prompt<'a>(&self, tab: &'a ShellTab) -> Option<&'a [String]> {
        let terminal = tab.terminal();
        let taken = terminal.on_alternate_screen() || terminal.modes.mouse_reports;
        match &terminal.shell {
            ShellState::Waiting(prompt) if self.on && tab.integrated() && !taken => Some(prompt),
            _ => None,
        }
    }
}

impl Core {
    /// Where the commands run in the editor are kept from now on. Before the
    /// shell is opened.
    pub fn keep_history(&mut self, store: Box<dyn CommandHistory>) {
        self.typing.store = Some(store);
    }

    /// Where `Tab` looks for names from now on.
    pub fn use_folders(&mut self, folders: Box<dyn Folders>) {
        self.typing.folders = Some(folders);
    }

    /// Turns the editor on, or off to write in the shell itself.
    pub fn toggle_editor(&mut self) -> Vec<WindowCommand> {
        self.typing.on = !self.typing.on;
        vec![WindowCommand::Redraw]
    }

    pub(super) fn editor_view(&self) -> Option<EditorView> {
        let prompt = self.typing.prompt(self.output.tab()?)?;
        let editor = &self.typing.editor;
        let letters = pixels_of_points(self.settings.font.size, self.scale);
        Some(EditorView {
            prompt: prompt.to_vec(),
            lines: editor.text().split('\n').map(str::to_string).collect(),
            cursor: editor.cursor(),
            suggestion: editor.suggestion().to_string(),
            refused: self.typing.refused,
            // A size of letters is far inside what these hold, never negative.
            letters: letters.round().max(0.0) as u32,
        })
    }

    /// A key goes to the editor while it is shown, and else to the shell.
    pub fn typed(&mut self, input: &KeyInput) -> Vec<WindowCommand> {
        // A key that writes something is a no to a paste that was asked about.
        let writes = input.key.is_some() || input.text.is_some();
        let warned = writes && self.pasting.forget();
        let mut commands = self.key_to_editor_or_shell(input);
        if warned && commands.is_empty() {
            commands = redraw_if(true);
        }
        commands
    }

    fn key_to_editor_or_shell(&mut self, input: &KeyInput) -> Vec<WindowCommand> {
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        if self.typing.prompt(tab).is_none() {
            let moved = tab.typed(input).unwrap_or_else(|error| {
                log_warn!("the key did not reach the shell: {error}");
                false
            });
            return redraw_if(moved);
        }
        let Some(key) = editor_key(input) else {
            return Vec::new();
        };
        match self.typing.editor.apply(key) {
            Outcome::Unchanged => Vec::new(),
            Outcome::Changed => {
                self.typing.refused = false;
                tab.to_screen();
                redraw_if(true)
            }
            Outcome::Interrupt => {
                if let Err(error) = tab.send(INTERRUPT) {
                    log_warn!("Ctrl+C did not reach the shell: {error}");
                }
                Vec::new()
            }
            Outcome::Run(command) => self.run(&command),
            Outcome::Complete(word) => {
                tab.to_screen();
                let current = tab.terminal().folder.as_deref();
                redraw_if(self.typing.complete(&word, current))
            }
        }
    }

    /// Sends the command of the editor to the shell. What cannot be sent whole
    /// stays written, and the editor says why.
    fn run(&mut self, command: &str) -> Vec<WindowCommand> {
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let modes = tab.terminal().modes;
        // Only a PowerShell has the integration so far.
        let Some(sent) = submission(command, modes, tab.integrated()) else {
            self.typing.refused = true;
            return redraw_if(true);
        };
        if let Err(error) = tab.send(&sent) {
            log_warn!("the command did not reach the shell: {error}");
            return Vec::new();
        }
        self.typing.refused = false;
        self.chosen_block = None;
        let kept = self.typing.remember(command);
        self.typing.editor.sent(kept);
        // An empty one only gets another prompt.
        if !command.trim().is_empty() {
            tab.running();
        }
        redraw_if(true)
    }
}
