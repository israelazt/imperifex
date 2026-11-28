//! The state of the window while it is open: what the interface is shown, and
//! what the program is asked to do with the real window.

mod clipboard;
mod output;
mod typing;

use imperifex_domain::draw::DrawList;
use imperifex_domain::i18n::{text, Language, Text};
use imperifex_domain::log_warn;
use imperifex_domain::ports::{SystemAppearance, Wake};
use imperifex_domain::settings::Settings;
use imperifex_domain::shell::tab_name;
use imperifex_domain::theme::{palette, resolve_theme, Theme};
use imperifex_domain::view::{Action, Edge, Model};

pub use output::Start;

/// What the program does with the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowCommand {
    Minimize,
    SetMaximized(bool),
    Close,
    /// Move it with the mouse.
    Drag,
    /// Change its size from that edge, with the mouse.
    Resize(Edge),
    /// The window's menu that the system draws, where the mouse is.
    ShowSystemMenu,
    /// What is shown is out of date.
    Redraw,
}

pub struct Core {
    settings: Settings,
    appearance: Box<dyn SystemAppearance>,
    theme: Theme,
    language: Language,
    /// Width and height in physical pixels.
    size: (u32, u32),
    scale: f32,
    maximized: bool,
    focused: bool,
    /// What was last drawn.
    shown: Option<DrawList>,
    output: output::Output,
    typing: typing::Typing,
    pasting: clipboard::Pasting,
}

impl Core {
    /// The window starts with no size: the program says it with
    /// [`Core::resized`] and [`Core::set_scale`] before the first frame.
    pub fn new(
        settings: Settings,
        language: Language,
        appearance: Box<dyn SystemAppearance>,
    ) -> Self {
        let theme = current_theme(&settings, appearance.as_ref());
        let typing = typing::Typing::new(settings.command_editor);
        Self {
            pasting: clipboard::Pasting::default(),
            typing,
            settings,
            appearance,
            theme,
            language,
            size: (0, 0),
            scale: 1.0,
            maximized: false,
            focused: true,
            shown: None,
            output: output::Output::default(),
        }
    }

    /// From now on `changed` is called when the system's theme may have changed;
    /// then [`Core::system_theme_changed`] is to be asked.
    pub fn watch_system(&self, changed: Wake) -> Result<(), String> {
        self.appearance.watch(changed)
    }

    pub fn model(&self) -> Model {
        Model {
            title: text(self.language, Text::AppName).into(),
            tab: self
                .output
                .tab()
                .map(|tab| tab_name(tab.line(), self.language)),
            language: self.language,
            palette: palette(self.theme),
            scale: self.scale,
            window: self.size,
            maximized: self.maximized,
            focused: self.focused,
            full_screen: self
                .output
                .tab()
                .is_some_and(|tab| tab.terminal().on_alternate_screen()),
            notice: self.notice(),
            editor: self.editor_view(),
            warning: self.warning(),
        }
    }

    /// What is told inside the window.
    fn notice(&self) -> Vec<String> {
        let Some(why) = self.output.failure() else {
            return Vec::new();
        };
        let said = text(self.language, Text::ShellNotStarted);
        vec![said.into(), why.into()]
    }

    pub fn resized(&mut self, size: (u32, u32), maximized: bool) {
        self.size = size;
        self.maximized = maximized;
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// What the user asked through the interface, as orders for the window.
    pub fn apply(&mut self, action: Action) -> Vec<WindowCommand> {
        if let Action::PlaceCursor { line, column } = action {
            self.typing.place_cursor(line, column);
            return redraw_if(true);
        }
        vec![match action {
            Action::Minimize => WindowCommand::Minimize,
            Action::ToggleMaximize => WindowCommand::SetMaximized(!self.maximized),
            Action::Close => WindowCommand::Close,
            Action::Drag => WindowCommand::Drag,
            Action::Resize(edge) => WindowCommand::Resize(edge),
            Action::SystemMenu => WindowCommand::ShowSystemMenu,
            Action::PlaceCursor { .. } => WindowCommand::Redraw,
        }]
    }

    pub fn system_theme_changed(&mut self) -> Vec<WindowCommand> {
        let theme = current_theme(&self.settings, self.appearance.as_ref());
        if theme == self.theme {
            return Vec::new();
        }
        self.theme = theme;
        vec![WindowCommand::Redraw]
    }

    /// Whether `list` differs from what the window shows: only then is there a
    /// reason to draw (T-D6).
    pub fn is_news(&self, list: &DrawList) -> bool {
        self.shown.as_ref() != Some(list)
    }

    /// `list` is now on the window.
    pub fn was_shown(&mut self, list: DrawList) {
        self.output.was_shown();
        self.shown = Some(list);
    }
}

fn redraw_if(changed: bool) -> Vec<WindowCommand> {
    if changed {
        vec![WindowCommand::Redraw]
    } else {
        Vec::new()
    }
}

fn current_theme(settings: &Settings, appearance: &dyn SystemAppearance) -> Theme {
    let dark = appearance
        .dark()
        .map_err(|e| log_warn!("the system theme is not known: {e}"))
        .ok();
    resolve_theme(settings.theme, dark)
}

#[cfg(test)]
mod tests;
