//! The shell of the window: when it is opened, when its output is read and what
//! of it is drawn.

use std::time::Duration;

use imperifex_domain::draw::Rect;
use imperifex_domain::font::{pixels_of_points, CellSize};
use imperifex_domain::grid::Damage;
use imperifex_domain::input::KeyInput;
use imperifex_domain::log_warn;
use imperifex_domain::picture::TerminalPicture;
use imperifex_domain::ports::{Clock, PtyLauncher, SequenceParser, Wake};
use imperifex_domain::theme::palette;
use imperifex_domain::view::Point;

use super::{Core, WindowCommand};
use crate::shell_tab::{Opening, ShellTab};

/// What opening the window's shell needs from the program.
pub struct Start<'a> {
    pub launcher: &'a dyn PtyLauncher,
    pub new_parser: &'a dyn Fn() -> Box<dyn SequenceParser>,
    /// Called when there is something to read: then [`Core::read_output`] is
    /// to be asked.
    pub wake: Wake,
}

#[derive(Default)]
enum Shell {
    #[default]
    NotOpened,
    Open(Box<ShellTab>),
    /// No shell started; why the last one did not.
    Failed(String),
}

#[derive(Default)]
pub(super) struct Output {
    shell: Shell,
    /// Where the grid goes and the size of its cells, as last drawn.
    place: Option<(Rect, CellSize)>,
    /// Output asked for a frame that is not on the window yet. Until it is, no
    /// more is read: a program that does not stop writing would leave the
    /// window no turn to draw.
    awaiting_frame: bool,
    /// There may be output left to read.
    unread: bool,
}

impl Output {
    pub fn tab(&self) -> Option<&ShellTab> {
        match &self.shell {
            Shell::Open(tab) => Some(tab),
            _ => None,
        }
    }

    pub(super) fn tab_mut(&mut self) -> Option<&mut ShellTab> {
        match &mut self.shell {
            Shell::Open(tab) => Some(tab),
            _ => None,
        }
    }

    pub fn failure(&self) -> Option<&str> {
        match &self.shell {
            Shell::Failed(why) => Some(why),
            _ => None,
        }
    }

    pub fn was_shown(&mut self) {
        self.awaiting_frame = false;
    }

    /// The cell of the grid at a place of the window, or the closest one.
    fn cell_at(&self, (x, y): Point) -> Option<(u16, u16)> {
        let (area, cell) = self.place?;
        let (columns, rows) = self.size()?;
        let along = |at: i32, start: i32, side: u32, count: u16| {
            let from = i64::from(at) - i64::from(start);
            let whole = from / i64::from(side.max(1));
            u16::try_from(whole.clamp(0, i64::from(count) - 1)).unwrap_or(0)
        };
        Some((
            along(x, area.x, cell.width, columns),
            along(y, area.y, cell.height, rows),
        ))
    }

    /// Columns and rows that fit where the grid goes.
    fn size(&self) -> Option<(u16, u16)> {
        let (area, cell) = self.place?;
        Some(cell.fit(area.width, area.height))
    }
}

fn redraw_if(changed: bool) -> Vec<WindowCommand> {
    if changed {
        vec![WindowCommand::Redraw]
    } else {
        Vec::new()
    }
}

impl Core {
    /// The size of the grid's letters in points, to measure its cell.
    pub fn font_points(&self) -> f32 {
        self.settings.font.size
    }

    /// The grid now goes in `area`, with cells of `cell`.
    pub fn fit(&mut self, area: Rect, cell: CellSize, now: Duration) {
        self.output.place = Some((area, cell));
        let size = self.output.size();
        if let (Some(size), Some(tab)) = (size, self.output.tab_mut()) {
            tab.room_for(size, now);
        }
    }

    /// Opens the shell with the size that fits. Only the first call after
    /// [`Core::fit`] tries. `true` if there is something new to show.
    pub fn open_shell(&mut self, start: &Start<'_>) -> bool {
        let Some(size) = self.output.size() else {
            return false;
        };
        if !matches!(self.output.shell, Shell::NotOpened) {
            return false;
        }
        let opened = ShellTab::open(&Opening {
            launcher: start.launcher,
            new_parser: start.new_parser,
            chosen: self.settings.shell.as_deref(),
            size,
            wake: start.wake.clone(),
        });
        self.output.shell = match opened {
            Ok(tab) => Shell::Open(Box::new(tab)),
            Err(why) => Shell::Failed(why),
        };
        true
    }

    /// With nothing seen, as when minimized, no frame is waited for.
    fn is_visible(&self) -> bool {
        self.size.0 > 0 && self.size.1 > 0
    }

    fn must_wait_for_the_frame(&self) -> bool {
        self.output.awaiting_frame && self.is_visible()
    }

    /// Reads what the shell wrote, unless a frame is still to be drawn: then it
    /// stays for later, and [`Core::has_unread`] says so after the frame. A
    /// shell that ends closes the window.
    pub fn read_output(&mut self, clock: &dyn Clock) -> Vec<WindowCommand> {
        if self.must_wait_for_the_frame() {
            self.output.unread = true;
            return Vec::new();
        }
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let read = tab.read(clock);
        self.output.unread = read.more;
        if read.ended {
            self.output.unread = false;
            return vec![WindowCommand::Close];
        }
        if !read.redraw {
            return Vec::new();
        }
        self.output.awaiting_frame = true;
        vec![WindowCommand::Redraw]
    }

    /// Whether [`Core::read_output`] is to be asked again without waiting for
    /// the shell.
    pub fn has_unread(&self) -> bool {
        self.output.unread && !self.must_wait_for_the_frame()
    }

    /// The frame could not be drawn: the output is not held back for it.
    pub fn not_shown(&mut self) {
        self.output.awaiting_frame = false;
    }

    /// Gives the shell its new size once it has stayed the same long enough.
    pub fn settle_size(&mut self, now: Duration) -> Vec<WindowCommand> {
        let changed = self
            .output
            .tab_mut()
            .is_some_and(|tab| tab.resize_if_settled(now));
        redraw_if(changed)
    }

    /// How long from `now` until [`Core::settle_size`] has work.
    pub fn settles_in(&self, now: Duration) -> Option<Duration> {
        self.output.tab()?.resize_in(now)
    }

    pub fn typed(&mut self, input: &KeyInput) -> Vec<WindowCommand> {
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let moved = tab.typed(input).unwrap_or_else(|error| {
            log_warn!("the key did not reach the shell: {error}");
            false
        });
        redraw_if(moved)
    }

    /// The wheel turned `notches`, up when positive, with the mouse at `at`.
    pub fn wheel(&mut self, notches: f32, at: Point) -> Vec<WindowCommand> {
        let Some(cell) = self.output.cell_at(at) else {
            return Vec::new();
        };
        let Some(tab) = self.output.tab_mut() else {
            return Vec::new();
        };
        let moved = tab.wheel(notches, cell).unwrap_or_else(|error| {
            log_warn!("the wheel did not reach the shell: {error}");
            false
        });
        redraw_if(moved)
    }

    /// Moves the view a screen back in the history, or forward.
    pub fn page(&mut self, back: bool) -> Vec<WindowCommand> {
        let moved = self.output.tab_mut().is_some_and(|tab| tab.page(back));
        redraw_if(moved)
    }

    /// The rows that changed since the last call, to draw them: it forgets
    /// them, so it is asked only right before drawing.
    pub fn take_damage(&mut self) -> Damage {
        match self.output.tab_mut() {
            Some(tab) => tab.terminal_mut().screen_mut().take_damage(),
            None => Damage::default(),
        }
    }

    /// What the renderer draws of the terminal, if there is one.
    pub fn picture<'a>(&'a self, damage: &'a Damage) -> Option<TerminalPicture<'a>> {
        let (area, cell) = self.output.place?;
        Some(TerminalPicture {
            terminal: self.output.tab()?.terminal(),
            damage,
            back: self.output.tab()?.back(),
            area,
            palette: palette(self.theme),
            adjust_contrast: self.settings.adjust_contrast,
            cell,
            letters: pixels_of_points(self.settings.font.size, self.scale),
            // One logical pixel; a scale is never negative.
            line: self.scale.round().max(1.0) as u32,
            focused: self.focused,
        })
    }
}
